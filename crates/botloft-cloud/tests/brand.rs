//! What the people see: the flame, the e-mails drawn and the pages (spec 27.2,
//! 27.3).

mod common;

use axum::http::{StatusCode, header};
use common::server;

#[tokio::test]
async fn the_flame_is_served_as_a_png_that_may_be_kept() {
    let s = server();
    let flame = s.get("/brand/flame.png").await;
    assert_eq!(flame.status, StatusCode::OK);
    assert_eq!(flame.headers[header::CONTENT_TYPE], "image/png");
    assert!(
        flame.headers[header::CACHE_CONTROL]
            .to_str()
            .expect("text")
            .contains("max-age")
    );
    assert_eq!(&flame.bytes[..8], b"\x89PNG\r\n\x1a\n");
}

#[tokio::test]
async fn the_sign_in_mail_is_text_and_a_drawn_page_with_the_same_link() {
    let s = server();
    s.ask("ana@exemplo.com", "1.1.1.1").await;
    let mail = s.outbox.sent().pop().expect("a mail");
    let (path, _) = s.last_link();
    let link = format!("http://127.0.0.1:8787{path}");
    assert!(mail.body.contains(&link));
    // In the HTML the address is escaped, and it is both the button and the text.
    let escaped = link.replace('&', "&amp;");
    assert_eq!(mail.html.matches(&format!("href=\"{escaped}\"")).count(), 2);
    assert!(mail.html.contains("http://127.0.0.1:8787/brand/flame.png"));
    assert!(mail.html.contains("Entrar no Botloft") && mail.html.contains("PC da Ana"));
    assert!(mail.html.contains("<html lang=\"pt-BR\">"));
}

#[tokio::test]
async fn the_mail_that_deletes_an_account_has_a_red_button_and_the_link() {
    let s = server();
    let ana = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    assert_eq!(
        s.authed("POST", "/v1/account/delete", &ana).await.status,
        StatusCode::ACCEPTED
    );
    let mail = s.outbox.sent().pop().expect("a mail");
    let (path, _) = s.last_link();
    assert!(mail.html.contains("bgcolor=\"#bd3529\""));
    assert!(mail.html.contains(&path.replace('&', "&amp;")));
    assert!(mail.body.contains(&path));
}

#[tokio::test]
async fn the_pages_may_load_the_flame_and_nothing_else() {
    let s = server();
    s.ask("ana@exemplo.com", "1.1.1.1").await;
    let (path, _) = s.last_link();
    let page = s.get(&path).await;
    assert_eq!(page.status, StatusCode::OK);
    let policy = page.headers[header::CONTENT_SECURITY_POLICY]
        .to_str()
        .expect("text");
    assert!(policy.contains("img-src 'self'") && policy.contains("default-src 'none'"));
    assert!(page.body.contains("src=\"/brand/flame.png\""));
    assert!(!page.body.contains("<script"));
}
