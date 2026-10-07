//! The pages behind the e-mail links (spec 27.3): a card with the flame, in
//! the colors and the type of the app, light or dark as the device is. The
//! link only shows a button; the sign-in is confirmed by the button, because
//! mail scanners open links on their own and must not sign anyone in.

use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};

use crate::html::escape;
use crate::mail_text::Locale;

struct Words {
    sign_title: &'static str,
    sign_ask: &'static str,
    sign_button: &'static str,
    sign_once: &'static str,
    done_title: &'static str,
    done_text: &'static str,
    done_hint: &'static str,
    gone_title: &'static str,
    gone_text: &'static str,
    del_title: &'static str,
    /// `{}` is where the e-mail address goes.
    del_ask: &'static str,
    del_warning: &'static str,
    del_button: &'static str,
    deleted_title: &'static str,
    deleted_text: &'static str,
}

fn words(locale: Locale) -> Words {
    match locale {
        Locale::En => Words {
            sign_title: "Sign in to Botloft",
            sign_ask: "Confirm that you are signing in on this computer:",
            sign_button: "Sign in",
            sign_once: "This link works once.",
            done_title: "Done",
            done_text: "You can go back to Botloft now.",
            done_hint: "You can close this tab.",
            gone_title: "This link no longer works",
            gone_text: "It was already used or it expired. Ask for a new one in Botloft.",
            del_title: "Delete your account",
            del_ask: "This deletes the account of {} and every copy saved in it.",
            del_warning: "It cannot be undone.",
            del_button: "Delete everything",
            deleted_title: "Account deleted",
            deleted_text: "Your account and its copies are deleted.",
        },
        Locale::PtBr => Words {
            sign_title: "Entrar no Botloft",
            sign_ask: "Confirme que você está entrando neste computador:",
            sign_button: "Entrar",
            sign_once: "Este link vale uma vez.",
            done_title: "Pronto",
            done_text: "Pode voltar ao Botloft.",
            done_hint: "Você já pode fechar esta aba.",
            gone_title: "Este link não vale mais",
            gone_text: "Ele já foi usado ou expirou. Peça outro no Botloft.",
            del_title: "Apagar sua conta",
            del_ask: "Isto apaga a conta de {} e todas as cópias guardadas nela.",
            del_warning: "Não dá para desfazer.",
            del_button: "Apagar tudo",
            deleted_title: "Conta apagada",
            deleted_text: "Sua conta e as cópias foram apagadas.",
        },
        Locale::Es => Words {
            sign_title: "Entrar en Botloft",
            sign_ask: "Confirma que estás entrando en este equipo:",
            sign_button: "Entrar",
            sign_once: "Este enlace sirve una vez.",
            done_title: "Listo",
            done_text: "Ya puedes volver a Botloft.",
            done_hint: "Puedes cerrar esta pestaña.",
            gone_title: "Este enlace ya no sirve",
            gone_text: "Ya se usó o caducó. Pide otro en Botloft.",
            del_title: "Borrar tu cuenta",
            del_ask: "Esto borra la cuenta de {} y todas las copias guardadas en ella.",
            del_warning: "No se puede deshacer.",
            del_button: "Borrar todo",
            deleted_title: "Cuenta borrada",
            deleted_text: "Tu cuenta y sus copias se han borrado.",
        },
    }
}

const CHECK: &str = r#"<svg viewBox="0 0 24 24" width="30" height="30" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5"/></svg>"#;
const ALERT: &str = r#"<svg viewBox="0 0 24 24" width="30" height="30" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M12 7.5v5.5M12 16.6h.01"/></svg>"#;

/// The same card for every page: the flame, the name, then `inner`.
fn shell(locale: Locale, title: &str, inner: &str) -> String {
    format!(
        r##"<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="color-scheme" content="light dark">
<meta name="robots" content="noindex">
<title>{title}</title>
<style>{CSS}</style>
</head>
<body>
<main>
<img class="flame" src="/brand/flame.png" alt="" width="88" height="88">
<p class="brand">Botloft</p>
{inner}
</main>
</body>
</html>
"##,
        lang = locale.tag(),
        title = escape(title),
    )
}

const CSS: &str = r#"
:root{color-scheme:light dark;--canvas:#f3f3f0;--panel:#fff;--line:#d8d8d2;--ink:#121212;--soft:#3b3b38;--muted:#5f5f59;--accent:#b83a14;--on-accent:#fff;--ok:#17703d;--danger:#bd3529}
@media (prefers-color-scheme:dark){:root{--canvas:#0b0b0b;--panel:#141414;--line:#262626;--ink:#eeeeea;--soft:#c6c6c0;--muted:#8c8c86;--accent:#ff7a59;--on-accent:#121212;--ok:#4fc382;--danger:#ff6d60}}
*{box-sizing:border-box}
body{margin:0;min-height:100vh;display:grid;place-items:center;padding:24px;background:var(--canvas);color:var(--ink);font:16px/1.55 "IBM Plex Sans","Segoe UI Variable Text","Segoe UI",system-ui,sans-serif}
main{width:100%;max-width:420px;padding:36px 32px 32px;text-align:center;background:var(--panel);border:1px solid var(--line);border-radius:20px;box-shadow:0 10px 36px rgb(0 0 0/.07)}
.flame{display:block;width:88px;height:88px;margin:0 auto}
.brand{margin:4px 0 24px;color:var(--muted);font-size:14px;font-weight:700;letter-spacing:.04em}
h1{margin:0 0 10px;font-size:24px;line-height:1.25}
p{margin:0 0 20px;color:var(--soft)}
.chip{display:inline-block;max-width:100%;margin:0 0 22px;padding:5px 14px;color:var(--ink);font-weight:600;overflow-wrap:anywhere;background:var(--canvas);border:1px solid var(--line);border-radius:999px}
form{margin:0}
button{width:100%;padding:14px 20px;font:inherit;font-weight:600;color:var(--on-accent);cursor:pointer;background:var(--accent);border:0;border-radius:12px}
button:hover{filter:brightness(1.07)}
button:focus-visible{outline:3px solid var(--accent);outline-offset:3px}
button.danger{background:var(--danger);color:#fff}
.small{margin:14px 0 0;font-size:13px;color:var(--muted)}
.mark{display:grid;place-items:center;width:56px;height:56px;margin:0 auto 14px;border-radius:50%}
.mark.ok{color:var(--ok);background:color-mix(in srgb,var(--ok) 16%,transparent)}
.mark.alert{color:var(--danger);background:color-mix(in srgb,var(--danger) 14%,transparent)}
.warn{margin:-8px 0 22px;font-weight:600;color:var(--danger)}
@media (max-width:480px){main{padding:28px 20px 24px}}
"#;

/// The page with the sign-in button. `device` is what the app sent: escaped.
fn confirm_html(locale: Locale, device: &str, code: &str) -> String {
    let w = words(locale);
    let inner = format!(
        r#"<h1>{title}</h1>
<p>{ask}</p>
<p class="chip">{device}</p>
<form method="post" action="/v1/login/confirm">
<input type="hidden" name="code" value="{code}">
<button>{button}</button>
</form>
<p class="small">{once}</p>"#,
        title = w.sign_title,
        ask = w.sign_ask,
        device = escape(device),
        code = escape(code),
        button = w.sign_button,
        once = w.sign_once,
    );
    shell(locale, w.sign_title, &inner)
}

fn done_html(locale: Locale) -> String {
    let w = words(locale);
    let inner = format!(
        r#"<div class="mark ok">{CHECK}</div>
<h1>{title}</h1>
<p>{text}</p>
<p class="small">{hint}</p>"#,
        title = w.done_title,
        text = w.done_text,
        hint = w.done_hint,
    );
    shell(locale, w.done_title, &inner)
}

fn gone_html(locale: Locale) -> String {
    let w = words(locale);
    let inner = format!(
        r#"<div class="mark alert">{ALERT}</div>
<h1>{title}</h1>
<p>{text}</p>"#,
        title = w.gone_title,
        text = w.gone_text,
    );
    shell(locale, w.gone_title, &inner)
}

fn delete_html(locale: Locale, email: &str, code: &str) -> String {
    let w = words(locale);
    let inner = format!(
        r#"<h1>{title}</h1>
<p>{ask}</p>
<p class="warn">{warning}</p>
<form method="post" action="/v1/account/delete/confirm">
<input type="hidden" name="code" value="{code}">
<button class="danger">{button}</button>
</form>"#,
        title = w.del_title,
        ask = w
            .del_ask
            .replace("{}", &format!("<b>{}</b>", escape(email))),
        warning = w.del_warning,
        code = escape(code),
        button = w.del_button,
    );
    shell(locale, w.del_title, &inner)
}

fn deleted_html(locale: Locale) -> String {
    let w = words(locale);
    let inner = format!(
        r#"<div class="mark ok">{CHECK}</div>
<h1>{title}</h1>
<p>{text}</p>"#,
        title = w.deleted_title,
        text = w.deleted_text,
    );
    shell(locale, w.deleted_title, &inner)
}

/// The button page. The device name is what the app sent: it is escaped.
pub fn confirm(locale: Locale, device: &str, code: &str) -> Response {
    page(confirm_html(locale, device, code))
}

/// The button page for deleting an account.
pub fn delete_ask(locale: Locale, email: &str, code: &str) -> Response {
    page(delete_html(locale, email, code))
}

pub fn done(locale: Locale) -> Response {
    page(done_html(locale))
}

pub fn deleted(locale: Locale) -> Response {
    page(deleted_html(locale))
}

pub fn gone(locale: Locale) -> Response {
    let mut response = page(gone_html(locale));
    *response.status_mut() = StatusCode::GONE;
    response
}

fn page(html: String) -> Response {
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
            "default-src 'none'; style-src 'unsafe-inline'; img-src 'self'; form-action 'self'",
        ),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Locale; 3] = [Locale::En, Locale::PtBr, Locale::Es];

    #[test]
    fn every_page_has_the_flame_its_language_and_no_script() {
        for locale in ALL {
            for html in [
                confirm_html(locale, "PC", "c"),
                done_html(locale),
                gone_html(locale),
                delete_html(locale, "a@b.c", "c"),
                deleted_html(locale),
            ] {
                assert!(html.contains(r#"<img class="flame" src="/brand/flame.png""#));
                assert!(html.contains(&format!(r#"<html lang="{}">"#, locale.tag())));
                assert!(!html.contains("<script") && !html.contains("onclick"));
                assert!(html.contains("prefers-color-scheme:dark"));
            }
        }
    }

    #[test]
    fn the_buttons_post_to_their_own_address_with_the_code() {
        let sign = confirm_html(Locale::PtBr, "PC da Ana", "abc");
        assert!(sign.contains(r#"action="/v1/login/confirm""#));
        assert!(sign.contains(r#"name="code" value="abc""#));
        assert!(sign.contains("<button>Entrar</button>") && sign.contains("Entrar no Botloft"));
        let del = delete_html(Locale::En, "ana@exemplo.com", "abc");
        assert!(del.contains(r#"action="/v1/account/delete/confirm""#));
        assert!(del.contains(r#"<button class="danger">Delete everything</button>"#));
        assert!(del.contains("<b>ana@exemplo.com</b>"));
    }

    #[test]
    fn what_the_app_sent_cannot_become_markup() {
        let html = confirm_html(Locale::En, "<script>x()</script>", "\"><b>");
        assert!(html.contains("&lt;script&gt;x()&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
        assert!(html.contains(r#"value="&quot;&gt;&lt;b&gt;""#));
        let del = delete_html(Locale::En, "<i>@x.co", "c");
        assert!(del.contains("&lt;i&gt;@x.co") && !del.contains("<i>"));
    }

    /// The words the tests of the flow and the app look for.
    #[test]
    fn the_words_the_flow_relies_on_are_still_there() {
        assert!(done_html(Locale::PtBr).contains("Pode voltar ao Botloft"));
        assert!(gone_html(Locale::En).contains("Ask for a new one in Botloft"));
    }

    /// Writes every page into `$BOTLOFT_PREVIEW_DIR` to look at in a browser:
    /// `BOTLOFT_PREVIEW_DIR=... cargo test -p botloft-cloud preview -- --ignored`.
    #[test]
    #[ignore = "writes files to look at"]
    fn preview() {
        let dir = std::env::var("BOTLOFT_PREVIEW_DIR").expect("set BOTLOFT_PREVIEW_DIR");
        std::fs::create_dir_all(format!("{dir}/brand")).expect("dir");
        std::fs::copy(
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/flame.png"),
            format!("{dir}/brand/flame.png"),
        )
        .expect("flame");
        for (name, locale) in [("en", Locale::En), ("pt", Locale::PtBr), ("es", Locale::Es)] {
            let pages = [
                ("confirm", confirm_html(locale, "PC da Ana", "c0de")),
                ("done", done_html(locale)),
                ("gone", gone_html(locale)),
                ("delete", delete_html(locale, "ana@exemplo.com", "c0de")),
                ("deleted", deleted_html(locale)),
            ];
            for (page, html) in pages {
                std::fs::write(format!("{dir}/page-{page}-{name}.html"), html).expect("page");
            }
            let base = std::env::var("BOTLOFT_PREVIEW_BASE").unwrap_or_default();
            let link =
                format!("{base}/v1/login/confirm?code=Zk3pQxV8Lw2mN7aR9tYuB4cEhJ6dFgS1oIvX0nM5qWe");
            let mail = crate::mail_text::login_mail(locale, &link, "PC da Ana", &base);
            std::fs::write(format!("{dir}/mail-login-{name}.html"), mail.html).expect("mail");
            let gone = crate::mail_text::delete_mail(locale, &link, &base);
            std::fs::write(format!("{dir}/mail-delete-{name}.html"), gone.html).expect("mail");
        }
    }
}
