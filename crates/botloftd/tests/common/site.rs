//! A small web site on 127.0.0.1 for the browser tests: a form, the page
//! it leads to, a link that opens a new tab and a button that shows a
//! dialog.

use std::collections::HashMap;
use std::net::SocketAddr;

use axum::Router;
use axum::extract::Query;
use axum::response::Html;
use axum::routing::get;
use tokio::net::TcpListener;

const FORM: &str = "<!doctype html><title>Order</title><h1>Order a cake</h1>\
    <form action=\"/done\"><label>Name <input name=\"name\"></label>\
    <label>Size <select name=\"size\"><option>Small</option><option>Large</option></select></label>\
    <button>Send</button></form><p style=\"display:none\">Hidden text</p>";

const POPUP: &str = "<!doctype html><title>Links</title>\
    <a href=\"/done?name=Tab&size=Small\" target=\"_blank\">Open in a new tab</a>\
    <button onclick=\"alert('Hi there')\">Warn me</button>";

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
    let app = Router::new()
        .route("/", get(|| async { Html(FORM) }))
        .route("/done", get(done))
        .route("/links", get(|| async { Html(POPUP) }));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    addr
}
