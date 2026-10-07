//! Deleting an account (spec 27.3): asked by the app, confirmed by the button
//! in an e-mail, and then the account, its devices and every copy are gone.

use axum::Json;
use axum::extract::{Form, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use rusqlite::{Connection, OptionalExtension, params};
use serde::Deserialize;

use crate::accounts::LOGIN_TTL_MS;
use crate::app::AppState;
use crate::copy_rows;
use crate::error::ApiError;
use crate::limits::{self, GAP, PER_EMAIL};
use crate::mail_text::{Locale, delete_mail};
use crate::mailer::Mail;
use crate::session::Authed;
use crate::{pages, tokens};

#[derive(Deserialize, Default)]
pub struct Ask {
    #[serde(default)]
    locale: String,
}

/// `POST /v1/account/delete`: mails a link; nothing is deleted yet.
pub async fn ask(
    State(state): State<AppState>,
    Authed(who): Authed,
    body: Option<Json<Ask>>,
) -> Result<StatusCode, ApiError> {
    let locale = Locale::parse(&body.map(|Json(ask)| ask.locale).unwrap_or_default());
    let key = tokens::hash(&format!("delete:{}", who.account_id));
    let now = state.clock.now();
    let taken = state
        .db
        .run(|conn| limits::take(conn, now, &[(&PER_EMAIL, &key), (&GAP, &key)]))?;
    taken.map_err(ApiError::rate_limited)?;

    let code = tokens::random()?;
    state.db.run(|conn| {
        conn.execute(
            "INSERT INTO deletions (code_hash, account_id, locale, expires_at) \
             VALUES (?1, ?2, ?3, ?4)",
            params![
                tokens::hash(&code),
                who.account_id,
                locale.tag(),
                now + LOGIN_TTL_MS
            ],
        )
    })?;
    let link = format!(
        "{}/v1/account/delete/confirm?code={code}",
        state.config.base_url()
    );
    let (subject, text) = delete_mail(locale, &link);
    let sent = state
        .mailer
        .send(Mail {
            to: who.email,
            subject,
            body: text,
        })
        .await;
    if sent.is_err() {
        state.db.run(|conn| {
            conn.execute(
                "DELETE FROM deletions WHERE code_hash = ?1",
                [tokens::hash(&code)],
            )
        })?;
        return Err(ApiError::mail_failed());
    }
    Ok(StatusCode::ACCEPTED)
}

#[derive(Deserialize)]
pub struct Code {
    code: String,
}

fn waiting(
    conn: &Connection,
    code_hash: &str,
    now: i64,
) -> rusqlite::Result<Option<(i64, String, String)>> {
    conn.query_row(
        "SELECT deletions.account_id, deletions.locale, accounts.email FROM deletions \
         JOIN accounts ON accounts.id = deletions.account_id \
         WHERE deletions.code_hash = ?1 AND deletions.expires_at > ?2",
        params![code_hash, now],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
    .optional()
}

/// `GET /v1/account/delete/confirm?code=`: the page; only the button deletes.
pub async fn confirm_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(Code { code }): Query<Code>,
) -> Result<Response, ApiError> {
    let now = state.clock.now();
    let found = state
        .db
        .run(|conn| waiting(conn, &tokens::hash(&code), now))?;
    Ok(match found {
        Some((_, locale, email)) => pages::delete_ask(Locale::parse(&locale), &email, &code),
        None => pages::gone(Locale::of_request(&headers)),
    })
}

/// `POST /v1/account/delete/confirm`
pub async fn confirm(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(Code { code }): Form<Code>,
) -> Result<Response, ApiError> {
    let now = state.clock.now();
    let code_hash = tokens::hash(&code);
    let gone = state.db.run(|conn| {
        let Some((account_id, locale, _)) = waiting(conn, &code_hash, now)? else {
            return Ok(None);
        };
        let keys = copy_rows::keys(conn, account_id)?;
        conn.execute("DELETE FROM accounts WHERE id = ?1", [account_id])?;
        Ok(Some((Locale::parse(&locale), keys)))
    })?;
    let Some((locale, keys)) = gone else {
        return Ok(pages::gone(Locale::of_request(&headers)));
    };
    // The rows are gone; now the bytes.
    for key in keys {
        state.store.delete(&key).await;
    }
    Ok(pages::deleted(locale))
}
