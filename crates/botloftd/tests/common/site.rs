//! A small web site on 127.0.0.1 for the browser tests: a form, the page
//! it leads to, a link that opens a new tab, a button that shows a dialog,
//! a sign-in page with its fields at fixed points, for the owner's hands,
//! a page that says how big its window is, and one that counts its loads.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use axum::Form;
use axum::Router;
use axum::extract::Query;
use axum::response::Html;
use axum::routing::{get, post};
use tokio::net::TcpListener;

const FORM: &str = "<!doctype html><title>Order</title><h1>Order a cake</h1>\
    <form action=\"/done\"><label>Name <input name=\"name\"></label>\
    <label>Size <select name=\"size\"><option>Small</option><option>Large</option></select></label>\
    <button>Send</button></form><p style=\"display:none\">Hidden text</p>";

const POPUP: &str = "<!doctype html><title>Links</title>\
    <a href=\"/done?name=Tab&size=Small\" target=\"_blank\">Open in a new tab</a>\
    <button onclick=\"alert('Hi there')\">Warn me</button>";

/// The fields' middles: user (200, 115), password (200, 215).
const LOGIN: &str = "<!doctype html><title>Sign in</title><style>body{margin:0}\
    input{position:absolute;left:100px;width:200px;height:30px;margin:0;padding:0;\
    border:1px solid}</style><form method=\"post\" action=\"/account\">\
    <input name=\"user\" aria-label=\"User\" style=\"top:100px\">\
    <input name=\"password\" type=\"password\" aria-label=\"Password\" style=\"top:200px\">\
    <button style=\"position:absolute;top:300px;left:100px\">Sign in</button></form>";

const SIZE: &str = "<!doctype html><title>Size</title><p id=\"size\"></p><script>\
    const show = () => { document.getElementById('size').textContent = \
    'Window: ' + innerWidth + ' x ' + innerHeight; }; show(); \
    addEventListener('resize', show);</script>";

async fn account(Form(form): Form<HashMap<String, String>>) -> Html<String> {
    let user = form.get("user").cloned().unwrap_or_default();
    let length = form
        .get("password")
        .map_or(0, |password| password.chars().count());
    Html(format!(
        "<!doctype html><title>Account</title><h1>Welcome, {user}</h1>\
         <p>Password length: {length}</p>"
    ))
}

async fn done(Query(query): Query<HashMap<String, String>>) -> Html<String> {
    let name = query.get("name").cloned().unwrap_or_default();
    let size = query.get("size").cloned().unwrap_or_default();
    Html(format!(
        "<!doctype html><title>Thanks</title><h1>Thanks, {name}</h1><p>Size: {size}</p>\
         <a href=\"/\">Order again</a>"
    ))
}

/// Serves the site until the test ends.
pub async fn serve() -> SocketAddr {
    // `/visits` counts how many times it was loaded.
    let visits = Arc::new(AtomicU32::new(0));
    let app = Router::new()
        .route(
            "/visits",
            get(move || {
                let count = visits.fetch_add(1, Ordering::SeqCst) + 1;
                async move {
                    Html(format!(
                        "<!doctype html><title>Visits</title><p>Loaded {count} times</p>"
                    ))
                }
            }),
        )
        .route("/", get(|| async { Html(FORM) }))
        .route("/done", get(done))
        .route("/links", get(|| async { Html(POPUP) }))
        .route("/login", get(|| async { Html(LOGIN) }))
        .route("/size", get(|| async { Html(SIZE) }))
        .route("/account", post(account));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    addr
}
