//! The routes (spec 27.3, 27.4) and what every handler shares.

use std::sync::Arc;

use axum::Router;
use axum::routing::{delete, get, post};

use crate::clock::Clock;
use crate::config::Config;
use crate::db::Db;
use crate::mailer::Mailer;
use crate::{login, session};

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub mailer: Arc<Mailer>,
    pub clock: Clock,
    pub config: Arc<Config>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/login", post(login::ask))
        .route(
            "/v1/login/confirm",
            get(login::confirm_page).post(login::confirm),
        )
        .route("/v1/login/{request}", get(login::poll))
        .route("/v1/me", get(session::me))
        .route("/v1/logout", post(session::logout))
        .route("/v1/devices/{id}", delete(session::remove_device))
        .with_state(state)
}
