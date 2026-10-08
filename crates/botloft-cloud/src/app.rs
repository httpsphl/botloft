//! The routes (spec 27.3, 27.4) and what every handler shares.

use std::sync::Arc;

use axum::Router;
use axum::routing::{delete, get, post};

use crate::clock::Clock;
use crate::config::Config;
use crate::db::Db;
use crate::hub::Hub;
use crate::mailer::Mailer;
use crate::push::Pusher;
use crate::store::CopyStore;
use crate::{copies, login, pairing, phone, push_api, relay, removal, session};

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub mailer: Arc<Mailer>,
    pub clock: Clock,
    pub config: Arc<Config>,
    pub store: CopyStore,
    pub hub: Hub,
    pub push: Pusher,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/brand/flame.png", get(crate::html::flame))
        .route("/v1/login", post(login::ask))
        .route(
            "/v1/login/confirm",
            get(login::confirm_page).post(login::confirm),
        )
        .route("/v1/login/{request}", get(login::poll))
        .route("/v1/me", get(session::me))
        .route("/v1/logout", post(session::logout))
        .route("/v1/devices/{id}", delete(session::remove_device))
        .route("/m", get(phone::index))
        .route("/m/", get(phone::index))
        .route("/m/{*path}", get(phone::file))
        .route("/v1/push/key", get(push_api::key))
        .route(
            "/v1/push/subscribe",
            post(push_api::subscribe).delete(push_api::unsubscribe),
        )
        .route("/v1/relay", get(relay::relay))
        .route("/v1/pairings", post(pairing::open))
        .route(
            "/v1/pairings/{id}",
            get(pairing::status).delete(pairing::cancel),
        )
        .route("/v1/pairings/{id}/join", post(pairing::join))
        .route("/v1/pairings/{id}/accept", post(pairing::accept))
        .route("/v1/pairings/{id}/result", get(pairing::result))
        .route("/v1/copies", get(copies::list).put(copies::put))
        .route("/v1/copies/{id}", get(copies::get).delete(copies::remove))
        .route("/v1/account/delete", post(removal::ask))
        .route(
            "/v1/account/delete/confirm",
            get(removal::confirm_page).post(removal::confirm),
        )
        .with_state(state)
}
