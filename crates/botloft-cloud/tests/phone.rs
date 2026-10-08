//! The phone's page (spec 28.7): files from a folder, under a policy, and
//! nothing outside it.

mod common;

use axum::http::{StatusCode, header};
use common::{server, server_with};

fn folder() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("dir");
    std::fs::create_dir_all(dir.path().join("assets")).expect("assets");
    std::fs::write(
        dir.path().join("index.html"),
        "<!doctype html><title>Botloft</title>",
    )
    .expect("index");
    std::fs::write(dir.path().join("sw.js"), "// worker").expect("worker");
    std::fs::write(dir.path().join("manifest.webmanifest"), "{}").expect("manifest");
    std::fs::write(
        dir.path().join("assets").join("index-abc123.js"),
        "console.log(1)",
    )
    .expect("js");
    std::fs::write(
        dir.path().join("assets").join("font-abc123.woff2"),
        [0u8, 1, 2],
    )
    .expect("font");
    dir
}

fn header_of(reply: &common::Reply, name: header::HeaderName) -> String {
    reply
        .headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}

#[tokio::test]
async fn the_page_is_served_with_its_own_policy_and_the_right_caching() {
    let dir = folder();
    let path = dir.path().to_path_buf();
    let s = server_with(|config| config.phone_dir = Some(path));

    for address in ["/m", "/m/", "/m/some/screen"] {
        let page = s.get(address).await;
        assert_eq!(page.status, StatusCode::OK, "{address}");
        assert!(page.body.contains("<title>Botloft</title>"));
        assert_eq!(
            header_of(&page, header::CONTENT_TYPE),
            "text/html; charset=utf-8"
        );
        assert_eq!(header_of(&page, header::CACHE_CONTROL), "no-cache");
        let policy = header_of(&page, header::CONTENT_SECURITY_POLICY);
        assert!(policy.contains("default-src 'none'") && policy.contains("connect-src 'self'"));
        assert!(!policy.contains("unsafe-inline") && !policy.contains("unsafe-eval"));
        assert_eq!(header_of(&page, header::REFERRER_POLICY), "no-referrer");
        assert_eq!(header_of(&page, header::X_CONTENT_TYPE_OPTIONS), "nosniff");
    }

    let script = s.get("/m/assets/index-abc123.js").await;
    assert_eq!(
        header_of(&script, header::CONTENT_TYPE),
        "text/javascript; charset=utf-8"
    );
    assert_eq!(
        header_of(&script, header::CACHE_CONTROL),
        "public, max-age=31536000, immutable"
    );
    assert_eq!(
        header_of(
            &s.get("/m/assets/font-abc123.woff2").await,
            header::CONTENT_TYPE
        ),
        "font/woff2"
    );
    // The worker and the manifest are looked at every time.
    let worker = s.get("/m/sw.js").await;
    assert_eq!(header_of(&worker, header::CACHE_CONTROL), "no-cache");
    assert_eq!(
        header_of(
            &s.get("/m/manifest.webmanifest").await,
            header::CONTENT_TYPE
        ),
        "application/manifest+json"
    );
    // A file that is not there is not the page.
    assert_eq!(
        s.get("/m/assets/nothing.js").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn nothing_outside_the_folder_can_be_asked_for() {
    let dir = folder();
    let outside = dir.path().join("..").join("phone-test-secret.txt");
    std::fs::write(&outside, "secret").expect("secret");
    let path = dir.path().to_path_buf();
    let s = server_with(|config| config.phone_dir = Some(path));
    for address in [
        "/m/../phone-test-secret.txt",
        "/m/%2e%2e/phone-test-secret.txt",
        "/m/assets/..%2f..%2fphone-test-secret.txt",
        "/m/assets/%2e%2e/%2e%2e/phone-test-secret.txt",
        "/m/..%5cphone-test-secret.txt",
        "/m/C:/Windows/win.ini",
    ] {
        let reply = s.get(address).await;
        assert!(!reply.body.contains("secret"), "{address}");
        assert_ne!(reply.status, StatusCode::OK, "{address}");
    }
    std::fs::remove_file(outside).ok();
}

#[tokio::test]
async fn without_a_folder_there_is_no_page() {
    let s = server();
    for address in ["/m", "/m/", "/m/assets/x.js", "/m/sw.js"] {
        assert_eq!(
            s.get(address).await.status,
            StatusCode::NOT_FOUND,
            "{address}"
        );
    }
}
