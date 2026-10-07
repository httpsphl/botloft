//! The server's HTTP API (spec 27.3, 27.4), one call each. Nothing here knows
//! about the daemon: the address and the token come in as arguments.

use std::time::Duration;

use botloft_core::protocol::CloudCopy;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};

use super::CloudError;

/// Small calls give up after this; transfers only when they go quiet.
const CALL: Duration = Duration::from_secs(30);
const QUIET: Duration = Duration::from_secs(60);

pub struct Server {
    pub(super) http: Client,
    base: String,
}

pub struct Started {
    pub request: String,
    pub wait: u32,
}

pub enum Polled {
    Pending,
    Expired,
    Approved {
        token: String,
        email: String,
        device: String,
    },
}

pub struct Me {
    pub email: String,
    pub used: u64,
    pub quota: u64,
}

#[derive(Deserialize)]
struct Failure {
    reason: String,
    message: String,
}

impl Server {
    /// `url` must be https, or http on this computer (tests).
    pub fn new(url: &str) -> Result<Self, CloudError> {
        let base = url.trim().trim_end_matches('/').to_owned();
        if base.is_empty() {
            return Err(CloudError::no_server("no server is set up"));
        }
        let here = ["http://127.0.0.1", "http://localhost", "http://[::1]"]
            .iter()
            .any(|prefix| base.starts_with(prefix));
        if !base.starts_with("https://") && !here {
            return Err(CloudError::no_server("the server address must be https"));
        }
        let http = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .read_timeout(QUIET)
            .user_agent(concat!("botloftd/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|err| CloudError::other(err.to_string()))?;
        Ok(Self { http, base })
    }

    pub(super) fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    /// Sends it; a connection that fails is `offline`, an answer that is not
    /// a success is the server's own `reason`.
    pub(super) async fn send(&self, request: RequestBuilder) -> Result<Response, CloudError> {
        let response = request.send().await.map_err(|_| CloudError::offline())?;
        if response.status().is_success() {
            return Ok(response);
        }
        let status = response.status();
        let failure = response.json::<Failure>().await.ok();
        Err(match (failure, status) {
            (Some(failure), _) => CloudError::from_server(&failure.reason, failure.message),
            (None, StatusCode::UNAUTHORIZED) => {
                CloudError::from_server("unauthorized", String::new())
            }
            (None, status) => CloudError::other(format!("the server answered {status}")),
        })
    }

    async fn json(&self, request: RequestBuilder) -> Result<Value, CloudError> {
        let response = self.send(request.timeout(CALL)).await?;
        response
            .json()
            .await
            .map_err(|_| CloudError::other("the server's answer was not understood".to_owned()))
    }

    pub async fn login(
        &self,
        email: &str,
        device_name: &str,
        locale: &str,
    ) -> Result<Started, CloudError> {
        let body = json!({ "email": email, "device_name": device_name, "locale": locale });
        let answer = self
            .json(self.http.post(self.url("/v1/login")).json(&body))
            .await?;
        Ok(Started {
            request: text(&answer, "request")?,
            wait: answer["wait"].as_u64().unwrap_or(600) as u32,
        })
    }

    pub async fn poll(&self, request: &str) -> Result<Polled, CloudError> {
        let answer = self
            .json(self.http.get(self.url(&format!("/v1/login/{request}"))))
            .await?;
        Ok(match answer["status"].as_str() {
            Some("approved") => Polled::Approved {
                token: text(&answer, "token")?,
                email: text(&answer, "email")?,
                device: text(&answer, "device")?,
            },
            Some("pending") => Polled::Pending,
            _ => Polled::Expired,
        })
    }

    pub async fn me(&self, token: &str, timeout: Duration) -> Result<Me, CloudError> {
        let response = self
            .send(
                self.http
                    .get(self.url("/v1/me"))
                    .bearer_auth(token)
                    .timeout(timeout),
            )
            .await?;
        let answer: Value = response
            .json()
            .await
            .map_err(|_| CloudError::other("the server's answer was not understood".to_owned()))?;
        Ok(Me {
            email: text(&answer, "email")?,
            used: answer["used"].as_u64().unwrap_or(0),
            quota: answer["quota"].as_u64().unwrap_or(0),
        })
    }

    pub async fn logout(&self, token: &str) -> Result<(), CloudError> {
        self.send(
            self.http
                .post(self.url("/v1/logout"))
                .bearer_auth(token)
                .timeout(CALL),
        )
        .await
        .map(|_| ())
    }

    pub async fn copies(&self, token: &str) -> Result<Vec<CloudCopy>, CloudError> {
        let answer = self
            .json(self.http.get(self.url("/v1/copies")).bearer_auth(token))
            .await?;
        let list = answer["copies"].as_array().cloned().unwrap_or_default();
        Ok(list.iter().filter_map(|copy| copy_of(copy).ok()).collect())
    }

    pub async fn delete_copy(&self, token: &str, id: &str) -> Result<(), CloudError> {
        self.send(
            self.http
                .delete(self.url(&format!("/v1/copies/{id}")))
                .bearer_auth(token)
                .timeout(CALL),
        )
        .await
        .map(|_| ())
    }

    pub async fn delete_account(&self, token: &str, locale: &str) -> Result<(), CloudError> {
        self.send(
            self.http
                .post(self.url("/v1/account/delete"))
                .bearer_auth(token)
                .json(&json!({ "locale": locale }))
                .timeout(CALL),
        )
        .await
        .map(|_| ())
    }
}

fn text(answer: &Value, key: &str) -> Result<String, CloudError> {
    answer[key]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| CloudError::other("the server's answer was not understood".to_owned()))
}

/// A copy as the server lists it and answers an upload.
pub(super) fn copy_of(answer: &Value) -> Result<CloudCopy, CloudError> {
    Ok(CloudCopy {
        id: text(answer, "id")?,
        size: answer["size"].as_u64().unwrap_or(0),
        created: answer["created"].as_i64().unwrap_or(0),
    })
}
