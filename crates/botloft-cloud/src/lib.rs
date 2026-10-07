//! The Botloft cloud server (spec 27): accounts by e-mail link, and the sealed
//! backup copies each account keeps. It never opens a copy.

mod accounts;
mod app;
mod clock;
mod config;
mod copies;
mod copy_rows;
mod db;
mod error;
mod limits;
mod login;
mod mail_text;
mod mailer;
mod pages;
mod removal;
mod session;
mod store;
mod tokens;

pub use app::{AppState, router};
pub use clock::Clock;
pub use config::{Bucket, Config, Smtp, Storage};
pub use db::Db;
pub use error::ApiError;
pub use mailer::{Mail, Mailer, Outbox, SmtpMailer};
pub use store::CopyStore;
