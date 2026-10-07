//! The three small pages behind the e-mail link (spec 27.3). The link only
//! shows a button; the sign-in is confirmed by the button, because mail
//! scanners open links on their own and must not sign anyone in.

use axum::http::{HeaderValue, header};
use axum::response::{Html, IntoResponse, Response};

use crate::mail_text::Locale;

struct Words {
    title: &'static str,
    ask: &'static str,
    button: &'static str,
    done_title: &'static str,
    done: &'static str,
    gone_title: &'static str,
    gone: &'static str,
    del_title: &'static str,
    del_ask: &'static str,
    del_button: &'static str,
    deleted: &'static str,
}

fn words(locale: Locale) -> Words {
    match locale {
        Locale::En => Words {
            title: "Sign in to Botloft",
            ask: "Sign in to Botloft on",
            button: "Sign in",
            done_title: "Done",
            done: "You can go back to Botloft now.",
            gone_title: "This link no longer works",
            gone: "It was already used or it expired. Ask for a new one in Botloft.",
            del_title: "Delete your account",
            del_ask: "This deletes the account of {} and every copy saved in it. It cannot be undone.",
            del_button: "Delete everything",
            deleted: "Your account and its copies are deleted.",
        },
        Locale::PtBr => Words {
            title: "Entrar no Botloft",
            ask: "Entrar no Botloft em",
            button: "Entrar",
            done_title: "Pronto",
            done: "Pode voltar ao Botloft.",
            gone_title: "Este link não vale mais",
            gone: "Ele já foi usado ou expirou. Peça outro no Botloft.",
            del_title: "Apagar sua conta",
            del_ask: "Isto apaga a conta de {} e todas as cópias guardadas nela. Não dá para desfazer.",
            del_button: "Apagar tudo",
            deleted: "Sua conta e as cópias foram apagadas.",
        },
        Locale::Es => Words {
            title: "Entrar en Botloft",
            ask: "Entrar en Botloft en",
            button: "Entrar",
            done_title: "Listo",
            done: "Ya puedes volver a Botloft.",
            gone_title: "Este enlace ya no sirve",
            gone: "Ya se usó o caducó. Pide otro en Botloft.",
            del_title: "Borrar tu cuenta",
            del_ask: "Esto borra la cuenta de {} y todas las copias guardadas en ella. No se puede deshacer.",
            del_button: "Borrar todo",
            deleted: "Tu cuenta y sus copias se han borrado.",
        },
    }
}

/// The button page. The device name is what the app sent: it is escaped.
pub fn confirm(locale: Locale, device: &str, code: &str) -> Response {
    let w = words(locale);
    let body = format!(
        "<h1>{}</h1><p>{} <b>{}</b>.</p>\
         <form method=\"post\" action=\"/v1/login/confirm\">\
         <input type=\"hidden\" name=\"code\" value=\"{}\"><button>{}</button></form>",
        w.title,
        w.ask,
        escape(device),
        escape(code),
        w.button
    );
    page(locale, w.title, &body)
}

/// The button page for deleting an account.
pub fn delete_ask(locale: Locale, email: &str, code: &str) -> Response {
    let w = words(locale);
    let body = format!(
        "<h1>{}</h1><p>{}</p>         <form method=\"post\" action=\"/v1/account/delete/confirm\">         <input type=\"hidden\" name=\"code\" value=\"{}\"><button>{}</button></form>",
        w.del_title,
        w.del_ask
            .replace("{}", &format!("<b>{}</b>", escape(email))),
        escape(code),
        w.del_button
    );
    page(locale, w.del_title, &body)
}

pub fn deleted(locale: Locale) -> Response {
    let w = words(locale);
    page(
        locale,
        w.del_title,
        &format!("<h1>{}</h1><p>{}</p>", w.del_title, w.deleted),
    )
}

pub fn done(locale: Locale) -> Response {
    let w = words(locale);
    page(
        locale,
        w.done_title,
        &format!("<h1>{}</h1><p>{}</p>", w.done_title, w.done),
    )
}

pub fn gone(locale: Locale) -> Response {
    let w = words(locale);
    let mut response = page(
        locale,
        w.gone_title,
        &format!("<h1>{}</h1><p>{}</p>", w.gone_title, w.gone),
    );
    *response.status_mut() = axum::http::StatusCode::GONE;
    response
}

fn page(locale: Locale, title: &str, body: &str) -> Response {
    let html = format!(
        "<!doctype html><html lang=\"{}\"><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>{title}</title><style>\
         body{{font:16px system-ui,sans-serif;max-width:26rem;margin:4rem auto;padding:0 1rem}}\
         button{{font:inherit;padding:.6rem 1.4rem;cursor:pointer}}</style>{body}",
        locale.tag()
    );
    let mut response = Html(html).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'none'; style-src 'unsafe-inline'; form-action 'self'",
        ),
    );
    response
}

fn escape(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '&' => "&amp;".to_owned(),
            '<' => "&lt;".to_owned(),
            '>' => "&gt;".to_owned(),
            '"' => "&quot;".to_owned(),
            '\'' => "&#39;".to_owned(),
            c => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_the_app_sent_cannot_become_markup() {
        assert_eq!(
            escape("<b onclick=\"x\">&'"),
            "&lt;b onclick=&quot;x&quot;&gt;&amp;&#39;"
        );
    }
}
