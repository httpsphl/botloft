//! A minimal MCP client over raw HTTP/1.1, speaking 2026-07-28 the way
//! Claude Code does: every request carries its version in `_meta` and in
//! the mirrored headers.

use std::net::SocketAddr;

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub const MODERN: &str = "2026-07-28";

#[derive(Debug, Clone)]
pub struct Reply {
    pub status: u16,
    pub body: Value,
}

/// POSTs `body` to `/mcp` with the given headers and reads the answer.
pub async fn post(addr: SocketAddr, headers: &[(&str, &str)], body: &str) -> Reply {
    raw(addr, "POST", headers, body).await
}

pub async fn raw(addr: SocketAddr, method: &str, headers: &[(&str, &str)], body: &str) -> Reply {
    let mut stream = TcpStream::connect(addr).await.expect("connect");
    let mut head = format!(
        "{method} /mcp HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\n\
         Accept: application/json, text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes()).await.expect("write head");
    stream.write_all(body.as_bytes()).await.expect("write body");
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.expect("read");
    let text = String::from_utf8(response).expect("utf8");
    let (head, body) = text.split_once("\r\n\r\n").expect("http response");
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .expect("status");
    let body = if body.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(body).unwrap_or_else(|_| Value::String(body.to_owned()))
    };
    Reply { status, body }
}

/// A bot's MCP connection.
pub struct Mcp {
    pub addr: SocketAddr,
    pub token: String,
    next_id: u64,
}

impl Mcp {
    pub fn new(addr: SocketAddr, token: String) -> Self {
        Self {
            addr,
            token,
            next_id: 1,
        }
    }

    /// A 2026-07-28 request with correct metadata and headers.
    pub async fn request(&mut self, method: &str, mut params: Value) -> Reply {
        let id = self.next_id;
        self.next_id += 1;
        params["_meta"] = json!({
            "io.modelcontextprotocol/protocolVersion": MODERN,
            "io.modelcontextprotocol/clientInfo": { "name": "tests", "version": "0" },
            "io.modelcontextprotocol/clientCapabilities": {},
        });
        let name = params["name"].as_str().map(str::to_owned);
        let body = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        let auth = format!("Bearer {}", self.token);
        let mut headers = vec![
            ("Authorization", auth.as_str()),
            ("MCP-Protocol-Version", MODERN),
            ("Mcp-Method", method),
        ];
        if let Some(name) = name.as_deref() {
            headers.push(("Mcp-Name", name));
        }
        post(self.addr, &headers, &body.to_string()).await
    }

    /// Calls a tool and returns its whole result.
    pub async fn tool_result(&mut self, name: &str, arguments: Value) -> Value {
        let reply = self
            .request(
                "tools/call",
                json!({ "name": name, "arguments": arguments }),
            )
            .await;
        assert_eq!(reply.status, 200, "{:?}", reply.body);
        reply.body["result"].clone()
    }

    /// Calls a tool that answers in plain text: `Ok(text)`, or `Err(text)`
    /// when it reports an error.
    pub async fn tool_text(&mut self, name: &str, arguments: Value) -> Result<String, String> {
        let result = self.tool_result(name, arguments).await;
        let text = result["content"]
            .as_array()
            .and_then(|parts| parts.iter().find(|part| part["type"] == "text"))
            .and_then(|part| part["text"].as_str())
            .expect("text content")
            .to_owned();
        if result["isError"] == json!(true) {
            Err(text)
        } else {
            Ok(text)
        }
    }

    /// Calls a tool: `Ok(output)` or `Err(message)` when it reports an error.
    pub async fn tool(&mut self, name: &str, arguments: Value) -> Result<Value, String> {
        let reply = self
            .request(
                "tools/call",
                json!({ "name": name, "arguments": arguments }),
            )
            .await;
        assert_eq!(reply.status, 200, "{:?}", reply.body);
        let result = &reply.body["result"];
        let text = result["content"][0]["text"]
            .as_str()
            .expect("text content")
            .to_owned();
        if result["isError"] == json!(true) {
            Err(text)
        } else {
            Ok(serde_json::from_str(&text).expect("JSON output"))
        }
    }
}
