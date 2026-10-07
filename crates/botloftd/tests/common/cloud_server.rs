//! The account server (`botloft-cloud`) running in the test's process on
//! 127.0.0.1, and a daemon signed in to it (spec 27).

use std::sync::Arc;

use botloft_cloud::{AppState, Clock, Config, CopyStore, Db, Mailer, Outbox, router};
use serde_json::json;
use tokio::net::TcpListener;

use super::{Client, TestDaemon};

/// The server of the account, and what a test needs to reach into.
pub struct CloudServer {
    pub url: String,
    pub outbox: Outbox,
    pub clock: Clock,
}

pub async fn cloud_server() -> CloudServer {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let url = format!("http://{}", listener.local_addr().expect("addr"));
    let (outbox, clock) = (Outbox::default(), Clock::default());
    let mut config = Config::for_tests();
    config.public_url = url.clone();
    let state = AppState {
        db: Db::memory().expect("db"),
        mailer: Arc::new(Mailer::Outbox(outbox.clone())),
        clock: clock.clone(),
        config: Arc::new(config),
        store: CopyStore::memory(),
    };
    tokio::spawn(async move {
        axum::serve(listener, router(state)).await.expect("serve");
    });
    CloudServer { url, outbox, clock }
}

impl CloudServer {
    /// The owner opens the link in the newest e-mail and presses the button.
    pub async fn press_the_link(&self) {
        let mail = self.outbox.sent().pop().expect("an e-mail");
        let link = mail
            .body
            .split_whitespace()
            .find(|word| word.starts_with("http"))
            .expect("a link");
        let code = link.split("code=").nth(1).expect("a code");
        let pressed = reqwest::Client::new()
            .post(format!("{}/v1/login/confirm", self.url))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(format!("code={code}"))
            .send()
            .await
            .expect("press");
        assert!(pressed.status().is_success(), "{}", pressed.status());
    }
}

/// A daemon that signed in to `cloud` as ana@exemplo.com.
pub async fn signed_in(cloud: &CloudServer) -> (TestDaemon, Client) {
    let t = TestDaemon::start_with_cloud(&cloud.url).await;
    let mut app = t.session().await;
    sign_in(cloud, &mut app).await;
    (t, app)
}

pub async fn sign_in(cloud: &CloudServer, app: &mut Client) {
    app.call(
        "cloud.signin",
        json!({ "email": "ana@exemplo.com", "locale": "pt-BR" }),
    )
    .await
    .expect("signin");
    cloud.press_the_link().await;
    let signed = app.notification("cloud.signed_in").await;
    assert_eq!(signed["email"], "ana@exemplo.com");
}
