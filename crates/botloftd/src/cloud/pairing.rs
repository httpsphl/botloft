//! The server's calls for connecting a phone (spec 28.3) and the address of
//! its relay (28.4). One call each; the pairing's meaning is in `mobile`.

use serde_json::json;

use super::CloudError;
use super::client::{CALL, Server};

/// What the computer finds when it looks at its pairing.
pub enum Seen {
    Waiting,
    Joined {
        name: String,
        phone_pub: String,
        proof: String,
    },
    /// Unknown, expired or cancelled.
    Gone,
}

impl Server {
    /// The relay's WebSocket address: `https` becomes `wss`.
    pub fn relay_url(&self) -> String {
        let url = self.url("/v1/relay");
        match url.strip_prefix("https://") {
            Some(rest) => format!("wss://{rest}"),
            None => url.replacen("http://", "ws://", 1),
        }
    }

    /// The address a phone's QR code opens (the part after `#` is added by
    /// the caller).
    pub fn phone_page(&self) -> String {
        self.url("/m")
    }

    /// Opens a pairing for the QR code. The server's `expires_in` is seconds.
    pub async fn open_pairing(&self, token: &str, id: &str) -> Result<u32, CloudError> {
        let answer = self
            .json(
                self.http
                    .post(self.url("/v1/pairings"))
                    .bearer_auth(token)
                    .json(&json!({ "id": id })),
            )
            .await?;
        Ok(answer["expires_in"].as_u64().unwrap_or(300) as u32)
    }

    pub async fn pairing(&self, token: &str, id: &str) -> Result<Seen, CloudError> {
        let answer = self
            .json(
                self.http
                    .get(self.url(&format!("/v1/pairings/{id}")))
                    .bearer_auth(token),
            )
            .await?;
        let field = |key: &str| answer[key].as_str().map(str::to_owned);
        Ok(match answer["status"].as_str() {
            Some("waiting") => Seen::Waiting,
            Some("joined") => match (field("phone_pub"), field("proof")) {
                (Some(phone_pub), Some(proof)) => Seen::Joined {
                    name: field("device_name").unwrap_or_default(),
                    phone_pub,
                    proof,
                },
                _ => Seen::Gone,
            },
            _ => Seen::Gone,
        })
    }

    pub async fn accept_pairing(
        &self,
        token: &str,
        id: &str,
        daemon_pub: &str,
        proof2: &str,
    ) -> Result<(), CloudError> {
        self.send(
            self.http
                .post(self.url(&format!("/v1/pairings/{id}/accept")))
                .bearer_auth(token)
                .json(&json!({ "daemon_pub": daemon_pub, "proof2": proof2 }))
                .timeout(CALL),
        )
        .await
        .map(|_| ())
    }

    /// Cancels, or refuses the phone that joined. A pairing that is already
    /// gone is not a problem.
    pub async fn cancel_pairing(&self, token: &str, id: &str) -> Result<(), CloudError> {
        let result = self
            .send(
                self.http
                    .delete(self.url(&format!("/v1/pairings/{id}")))
                    .bearer_auth(token)
                    .timeout(CALL),
            )
            .await;
        match result {
            Err(err) if err.reason == "not_found" => Ok(()),
            other => other.map(|_| ()),
        }
    }

    /// Takes a device (a phone, here) off the account. Already gone is fine.
    pub async fn remove_device(&self, token: &str, id: &str) -> Result<(), CloudError> {
        let result = self
            .send(
                self.http
                    .delete(self.url(&format!("/v1/devices/{id}")))
                    .bearer_auth(token)
                    .timeout(CALL),
            )
            .await;
        match result {
            Err(err) if err.reason == "not_found" => Ok(()),
            other => other.map(|_| ()),
        }
    }
}
