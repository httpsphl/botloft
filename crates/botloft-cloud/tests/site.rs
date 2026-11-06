//! The public site (spec 39): a page per language from a folder, under a
//! policy, and the download links that always reach the newest installer.

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use botloft_cloud::Site;
use common::{server, server_with};

fn folder() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("dir");
    let root = dir.path();
    for (path, text) in [
        ("index.html", "<!doctype html><title>Botloft</title><p>en"),
        (
            "pt-BR/index.html",
            "<!doctype html><title>Botloft</title><p>pt-BR",
        ),
        (
            "es/index.html",
            "<!doctype html><title>Botloft</title><p>es",
        ),
        ("assets/site-abc123.js", "console.log(1)"),
        ("assets/hero-abc123.png", "png"),
        ("og.png", "png"),
    ] {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().expect("parent")).expect("folders");
        std::fs::write(file, text).expect("file");
    }
    dir
}

fn site(dir: &tempfile::TempDir, releases: &str) -> Site {
    Site {
        dir: dir.path().to_path_buf(),
        releases: releases.to_owned(),
    }
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
async fn each_language_is_a_page_under_the_site_policy() {
    let dir = folder();
    let config = site(&dir, "https://example.com/releases");
    let s = server_with(|c| c.site = Some(config));

    for (address, language) in [
        ("/", "en"),
        ("/index.html", "en"),
        ("/pt-BR/", "pt-BR"),
        ("/pt-BR", "pt-BR"),
        ("/es/", "es"),
    ] {
        let page = s.get(address).await;
        assert_eq!(page.status, StatusCode::OK, "{address}");
        assert!(page.body.ends_with(&format!("<p>{language}")), "{address}");
        assert_eq!(
            header_of(&page, header::CONTENT_TYPE),
            "text/html; charset=utf-8"
        );
        assert_eq!(header_of(&page, header::CACHE_CONTROL), "no-cache");
        let policy = header_of(&page, header::CONTENT_SECURITY_POLICY);
        assert!(policy.contains("default-src 'none'") && policy.contains("script-src 'self'"));
        assert!(!policy.contains("unsafe-inline") && !policy.contains("unsafe-eval"));
        assert_eq!(header_of(&page, header::X_CONTENT_TYPE_OPTIONS), "nosniff");
    }

    let script = s.get("/assets/site-abc123.js").await;
    assert_eq!(
        header_of(&script, header::CACHE_CONTROL),
        "public, max-age=31536000, immutable"
    );
    assert_eq!(
        header_of(&s.get("/og.png").await, header::CONTENT_TYPE),
        "image/png"
    );
    // The routes of the account still answer as before.
    assert_eq!(s.get("/health").await.body, "ok");
    assert_eq!(s.get("/v1/me").await.status, StatusCode::UNAUTHORIZED);
    for missing in ["/fr/", "/assets/nothing.js", "/v1/nothing", "/pt%2DBR/"] {
        assert_eq!(
            s.get(missing).await.status,
            StatusCode::NOT_FOUND,
            "{missing}"
        );
    }
    let posted = s
        .send(
            Request::post("/pt-BR/")
                .body(Body::empty())
                .expect("request"),
        )
        .await;
    assert_eq!(posted.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn nothing_outside_the_site_can_be_asked_for() {
    let dir = folder();
    let outside = dir.path().join("..").join("site-test-secret.txt");
    std::fs::write(&outside, "secret").expect("secret");
    let config = site(&dir, "https://example.com/releases");
    let s = server_with(|c| c.site = Some(config));
    for address in [
        "/../site-test-secret.txt",
        "/%2e%2e/site-test-secret.txt",
        "/assets/..%2f..%2fsite-test-secret.txt",
        "/..%5csite-test-secret.txt",
        "/C:/Windows/win.ini",
    ] {
        let reply = s.get(address).await;
        assert!(!reply.body.contains("secret"), "{address}");
        assert_ne!(reply.status, StatusCode::OK, "{address}");
    }
    std::fs::remove_file(outside).ok();
}

#[tokio::test]
async fn without_a_site_there_is_no_page_and_no_download() {
    let s = server();
    for address in ["/", "/pt-BR/", "/download/windows"] {
        assert_eq!(
            s.get(address).await.status,
            StatusCode::NOT_FOUND,
            "{address}"
        );
    }
}

/// A releases page that sends `/latest` to `answer` and counts the asks.
async fn releases_page(answer: Option<&'static str>) -> (String, Arc<AtomicUsize>) {
    let asked = Arc::new(AtomicUsize::new(0));
    let counter = asked.clone();
    let app = Router::new().route(
        "/o/r/releases/latest",
        get(move || {
            counter.fetch_add(1, Ordering::SeqCst);
            async move {
                match answer {
                    Some(location) => {
                        (StatusCode::FOUND, [(header::LOCATION, location)]).into_response()
                    }
                    None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
                }
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (format!("http://{address}/o/r/releases"), asked)
}

#[tokio::test]
async fn a_download_goes_to_the_newest_installer_and_asks_only_now_and_then() {
    let (base, asked) = releases_page(Some("https://github.com/o/r/releases/tag/v0.19.0")).await;
    let dir = folder();
    let config = site(&dir, &base);
    let s = server_with(|c| c.site = Some(config));

    for (system, file) in [
        ("windows", "Botloft_0.19.0_x64-setup.exe"),
        ("macos", "Botloft_0.19.0_aarch64.dmg"),
        ("linux-deb", "Botloft_0.19.0_amd64.deb"),
        ("linux-appimage", "Botloft_0.19.0_amd64.AppImage"),
    ] {
        let reply = s.get(&format!("/download/{system}")).await;
        assert_eq!(reply.status, StatusCode::FOUND, "{system}");
        assert_eq!(
            header_of(&reply, header::LOCATION),
            format!("{base}/download/v0.19.0/{file}")
        );
        assert_eq!(header_of(&reply, header::CACHE_CONTROL), "no-store");
    }
    assert_eq!(asked.load(Ordering::SeqCst), 1, "learned once");

    s.clock.advance(11 * 60 * 1000);
    s.get("/download/windows").await;
    assert_eq!(
        asked.load(Ordering::SeqCst),
        2,
        "asked again after ten minutes"
    );

    assert_eq!(s.get("/download/haiku").await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn without_an_answer_a_download_opens_the_releases_page() {
    for answer in [None, Some("https://github.com/o/r/releases/tag/nightly")] {
        let (base, asked) = releases_page(answer).await;
        let dir = folder();
        let config = site(&dir, &base);
        let s = server_with(|c| c.site = Some(config));
        let reply = s.get("/download/windows").await;
        assert_eq!(reply.status, StatusCode::FOUND);
        assert_eq!(
            header_of(&reply, header::LOCATION),
            format!("{base}/latest")
        );
        // A failed look waits a minute before the next one.
        s.get("/download/macos").await;
        assert_eq!(asked.load(Ordering::SeqCst), 1);
        s.clock.advance(61 * 1000);
        s.get("/download/macos").await;
        assert_eq!(asked.load(Ordering::SeqCst), 2);
    }
}
