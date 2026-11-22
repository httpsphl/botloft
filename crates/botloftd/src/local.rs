//! A command on this computer talking to the daemon that runs here, as the
//! owner (spec 13): `botloftd pair` (42.6) and `botloftd update` (42.11).

use std::time::Duration;

use anyhow::{Context, bail};
use botloft_core::protocol::{PROTOCOL_VERSION, method};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// How long the daemon may take to answer.
const WAIT: Duration = Duration::from_secs(10);

/// A connection that passed `session.hello` with the owner's token.
pub struct Owner {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: u64,
}

impl Owner {
    /// Connects to the daemon on 127.0.0.1:`port` as `client`.
    pub async fn connect(port: u16, owner_token: &str, client: &str) -> anyhow::Result<Self> {
        let url = format!("ws://127.0.0.1:{port}/rpc");
        let connect = tokio_tungstenite::connect_async(url.as_str());
        let (socket, _) = tokio::time::timeout(WAIT, connect)
            .await
            .context("Botloft did not answer")?
            .with_context(|| format!("Botloft is not running on port {port} here"))?;
        let mut owner = Self { socket, next_id: 1 };
        let hello = json!({
            "token": owner_token,
            "client": { "name": client, "version": env!("CARGO_PKG_VERSION") },
            "protocol": PROTOCOL_VERSION,
        });
        owner.call(method::SESSION_HELLO, hello).await?;
        Ok(owner)
    }

    /// Sends a request and waits for its answer, past any notification.
    pub async fn call(&mut self, name: &str, params: Value) -> anyhow::Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        let request = json!({ "jsonrpc": "2.0", "id": id, "method": name, "params": params });
        self.socket.send(Message::text(request.to_string())).await?;
        loop {
            let frame = tokio::time::timeout(WAIT, self.socket.next())
                .await
                .context("Botloft did not answer")?;
            let Some(frame) = frame else {
                bail!("Botloft closed the connection");
            };
            let Message::Text(text) = frame? else {
                continue;
            };
            let answer: Value = serde_json::from_str(text.as_str())?;
            if answer.get("id") != Some(&json!(id)) {
                continue;
            }
            if let Some(error) = answer.get("error") {
                let message = error["message"].as_str().unwrap_or("the request failed");
                bail!("Botloft refused: {message}");
            }
            return Ok(answer["result"].clone());
        }
    }

    pub async fn close(mut self) {
        let _ = self.socket.close(None).await;
    }
}
