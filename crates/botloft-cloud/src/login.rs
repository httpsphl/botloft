//! Signing in by e-mail link (spec 27.3): ask, confirm in the e-mail, collect.

use axum::extract::{Extension, Form, Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::ConnectInfo};
use serde::Deserialize;
use serde_json::{Value, json};
use std::net::SocketAddr;

use crate::accounts::{self, LOGIN_TTL_MS, NewLogin, Request};
use crate::app::AppState;
use crate::error::ApiError;
use crate::limits::{self, GAP, PER_ADDRESS, PER_EMAIL};
use crate::mail_text::{Locale, login_mail};
use crate::mailer::Mail;
use crate::{pages, tokens};

#[derive(Deserialize)]
pub struct Ask {
    email: String,
    #[serde(default)]
    device_name: String,
    #[serde(default)]
    locale: String,
}

/// `POST /v1/login`: always the same answer, whoever has an account.
pub async fn ask(
    State(state): State<AppState>,
    headers: HeaderMap,
    connection: Option<Extension<ConnectInfo<SocketAddr>>>,
    Json(ask): Json<Ask>,
) -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    let email = clean_email(&ask.email).ok_or_else(ApiError::bad_email)?;
    let device = clean_device(&ask.device_name);
    let locale = Locale::parse(&ask.locale);
    let address = client_address(&state, &headers, connection.map(|Extension(info)| info));
    let (email_key, address_key) = (tokens::hash(&email), tokens::hash(&address));
    let now = state.clock.now();

    let taken = state.db.run(|conn| {
        limits::take(
            conn,
            now,
            &[
                (&PER_EMAIL, &email_key),
                (&PER_ADDRESS, &address_key),
                (&GAP, &email_key),
            ],
        )
    })?;
    taken.map_err(ApiError::rate_limited)?;

    let (request, code) = (tokens::random()?, tokens::random()?);
    let request_hash = tokens::hash(&request);
    state.db.run(|conn| {
        accounts::purge(conn, now)?;
        accounts::insert_login(
            conn,
            &NewLogin {
                request_hash: &request_hash,
                code_hash: &tokens::hash(&code),
                email: &email,
                device_name: &device,
                locale: locale.tag(),
            },
            now,
        )
    })?;

    let link = format!("{}/v1/login/confirm?code={code}", state.config.base_url());
    let letter = login_mail(locale, &link, &device, state.config.base_url());
    let sent = state
        .mailer
        .send(Mail {
            to: email,
            subject: letter.subject,
            body: letter.text,
            html: letter.html,
        })
        .await;
    if sent.is_err() {
        state
            .db
            .run(|conn| accounts::drop_login(conn, &request_hash))?;
        return Err(ApiError::mail_failed());
    }
    Ok((
        axum::http::StatusCode::ACCEPTED,
        Json(json!({ "request": request, "wait": LOGIN_TTL_MS / 1000 })),
    ))
}

/// `GET /v1/login/<request>`: the app asks whether the link was opened.
pub async fn poll(
    State(state): State<AppState>,
    Path(request): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let now = state.clock.now();
    let request_hash = tokens::hash(&request);
    let found = state
        .db
        .run(|conn| accounts::read_request(conn, &request_hash, now))?;
    match found {
        Request::Gone => Ok(Json(json!({ "status": "expired" }))),
        Request::Pending => Ok(Json(json!({ "status": "pending" }))),
        Request::Approved(approved) => {
            let (token, device) = (
                tokens::random()?,
                format!("dev_{}", &tokens::random()?[..16]),
            );
            let token_hash = tokens::hash(&token);
            let finished = state.db.run(|conn| {
                accounts::finish_login(conn, &request_hash, &approved, &device, &token_hash, now)
            })?;
            if !finished {
                // Two polls at once: the other one took it.
                return Ok(Json(json!({ "status": "expired" })));
            }
            Ok(Json(json!({
                "status": "approved",
                "token": token,
                "device": device,
                "email": approved.email,
            })))
        }
    }
}

#[derive(Deserialize)]
pub struct Code {
    code: String,
}

/// `GET /v1/login/confirm?code=`: the page the e-mail link opens. It only
/// asks; nothing is confirmed until the button posts.
pub async fn confirm_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(Code { code }): Query<Code>,
) -> Result<Response, ApiError> {
    let now = state.clock.now();
    let waiting = state
        .db
        .run(|conn| accounts::peek_login(conn, &tokens::hash(&code), now))?;
    Ok(match waiting {
        Some(waiting) => {
            pages::confirm(Locale::parse(&waiting.locale), &waiting.device_name, &code)
        }
        None => pages::gone(Locale::of_request(&headers)),
    })
}

/// `POST /v1/login/confirm`: the button.
pub async fn confirm(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(Code { code }): Form<Code>,
) -> Result<Response, ApiError> {
    let now = state.clock.now();
    let code_hash = tokens::hash(&code);
    let locale = state.db.run(|conn| {
        let waiting = accounts::peek_login(conn, &code_hash, now)?;
        let Some(waiting) = waiting else {
            return Ok(None);
        };
        Ok(accounts::confirm_login(conn, &code_hash, now)?.then(|| Locale::parse(&waiting.locale)))
    })?;
    Ok(match locale {
        Some(locale) => pages::done(locale).into_response(),
        None => pages::gone(Locale::of_request(&headers)),
    })
}

/// Where the request really comes from: the header the config names, if it
/// holds an address; else, behind a proxy, the last `X-Forwarded-For`; else
/// the connection.
pub(crate) fn client_address(
    state: &AppState,
    headers: &HeaderMap,
    connection: Option<ConnectInfo<SocketAddr>>,
) -> String {
    if let Some(name) = &state.config.client_ip_header
        && let Some(address) = headers
            .get(name.trim())
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse::<std::net::IpAddr>().ok())
    {
        return address.to_string();
    }
    if state.config.behind_proxy
        && let Some(forwarded) = headers
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|list| list.rsplit(',').next())
    {
        return forwarded.trim().to_owned();
    }
    connection.map_or_else(
        || "unknown".to_owned(),
        |ConnectInfo(addr)| addr.ip().to_string(),
    )
}

/// The address as it is stored: trimmed, lower case, one `@`, a domain with a
/// dot, and none of the characters that could break a header or a link.
fn clean_email(raw: &str) -> Option<String> {
    let email = raw.trim().to_lowercase();
    let (local, domain) = email.split_once('@')?;
    let bad = |c: char| c.is_whitespace() || c.is_control() || "<>()[]\\,;:\"@".contains(c);
    let ok = email.len() <= 254
        && !local.is_empty()
        && local.len() <= 64
        && !local.contains(bad)
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
        && !domain.contains(bad);
    ok.then_some(email)
}

/// The name of the computer: printable, at most 60 characters.
pub(crate) fn clean_device(raw: &str) -> String {
    let name: String = raw
        .chars()
        .filter(|c| !c.is_control())
        .take(60)
        .collect::<String>()
        .trim()
        .to_owned();
    if name.is_empty() {
        "Botloft".to_owned()
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_are_cleaned_or_refused() {
        assert_eq!(
            clean_email("  Ana@Exemplo.COM "),
            Some("ana@exemplo.com".to_owned())
        );
        for bad in [
            "",
            "ana",
            "ana@",
            "@x.com",
            "a b@x.com",
            "a@b",
            "a@@b.com",
            "a@b..com",
            "a\"@b.com",
            "a@b.com\r\nBcc: x@y.z",
            "<a>@b.com",
        ] {
            assert_eq!(clean_email(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn a_device_name_is_printable_and_short() {
        assert_eq!(clean_device(""), "Botloft");
        assert_eq!(clean_device("  PC\u{7} da Ana "), "PC da Ana");
        assert_eq!(clean_device(&"x".repeat(100)).len(), 60);
    }
}
