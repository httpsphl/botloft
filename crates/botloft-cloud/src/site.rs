//! The public site (spec 39): the files of `app/dist-site`, served at `/`.
//! Each language is a page made when the site is built (`/`, `/pt-BR/`,
//! `/es/`), so the server only hands out files, under a policy that lets
//! the page load nothing but its own.

use std::path::{Path, PathBuf};

use axum::extract::State;
use axum::http::{Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::app::AppState;
use crate::statics::send;

/// `[site]` in `cloud.toml`. Without it there is no site at `/` and no
/// `/download`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Site {
    /// The built files of `app/dist-site`.
    pub dir: PathBuf,
    /// The releases page whose newest installers `/download/<system>` hands
    /// out (download.rs).
    #[serde(default = "default_releases")]
    pub releases: String,
}

fn default_releases() -> String {
    "https://github.com/httpsphl/botloft/releases".to_owned()
}

/// What the site may load: its own files. It talks to no one.
const POLICY: &str = "default-src 'none'; script-src 'self'; style-src 'self'; \
img-src 'self' data:; font-src 'self'; connect-src 'self'; manifest-src 'self'; \
base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

fn dir(state: &AppState) -> Option<&Path> {
    state.config.site.as_ref().map(|site| site.dir.as_path())
}

/// `GET /`
pub async fn index(State(state): State<AppState>) -> Response {
    send(dir(&state), "index.html", POLICY).await
}

/// Any other address no route took: a file of the site, where a path with
/// no file extension is a folder with its page (`/pt-BR/` is
/// `pt-BR/index.html`). An escaped character never names one of its files.
pub async fn other(State(state): State<AppState>, method: Method, uri: Uri) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return StatusCode::NOT_FOUND.into_response();
    }
    let path = uri.path().trim_start_matches('/');
    if path.contains('%') {
        return StatusCode::NOT_FOUND.into_response();
    }
    let relative = page_of(path);
    send(dir(&state), &relative, POLICY).await
}

/// The file an address names: a folder stands for its `index.html`.
fn page_of(path: &str) -> String {
    if Path::new(path).extension().is_some() {
        return path.to_owned();
    }
    match path.trim_end_matches('/') {
        "" => "index.html".to_owned(),
        folder => format!("{folder}/index.html"),
    }
}

#[cfg(test)]
mod tests {
    use super::page_of;

    #[test]
    fn a_folder_stands_for_its_page() {
        assert_eq!(page_of("pt-BR/"), "pt-BR/index.html");
        assert_eq!(page_of("es"), "es/index.html");
        assert_eq!(page_of("assets/index-abc.js"), "assets/index-abc.js");
        assert_eq!(page_of("og.png"), "og.png");
        assert_eq!(page_of(""), "index.html");
    }
}
