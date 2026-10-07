//! The HTML of the e-mails (spec 27.2): one card, the flame, a title, a few
//! words, a big button and the link as text for whoever cannot press it.
//!
//! E-mail clients draw HTML their own way, so it is a table with inline
//! styles, nothing that needs a script or a web font, and a dark version for
//! the clients that ask for one. The text version goes in the same message.

use crate::html::escape;

/// What one e-mail says.
pub struct Card<'a> {
    pub lang: &'a str,
    /// What an inbox shows next to the subject.
    pub preheader: &'a str,
    pub title: &'a str,
    pub intro: &'a str,
    pub button: &'a str,
    /// The button is red: what it does cannot be undone.
    pub danger: bool,
    pub link: &'a str,
    pub fallback: &'a str,
    pub note: &'a str,
    pub device_line: &'a str,
    pub footer: &'a str,
    /// The address of the flame, whole, because a mail client has no site.
    pub flame_url: &'a str,
}

pub fn render(card: &Card<'_>) -> String {
    let (button, button_dark, button_ink_dark) = if card.danger {
        ("#bd3529", "#ff6d60", "#121212")
    } else {
        ("#b83a14", "#ff7a59", "#121212")
    };
    format!(
        r##"<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="color-scheme" content="light dark">
<meta name="supported-color-schemes" content="light dark">
<title>{title}</title>
<style>
@media (prefers-color-scheme: dark) {{
  .bg {{ background: #0b0b0b !important; }}
  .card {{ background: #141414 !important; border-color: #262626 !important; }}
  .ink {{ color: #eeeeea !important; }}
  .soft {{ color: #c6c6c0 !important; }}
  .muted {{ color: #8c8c86 !important; }}
  .btn {{ background: {button_dark} !important; }}
  .btn a {{ color: {button_ink_dark} !important; }}
  .url {{ background: #1c1c1c !important; border-color: #262626 !important; color: #c6c6c0 !important; }}
}}
@media (max-width: 480px) {{
  .card {{ padding: 24px 20px !important; }}
}}
</style>
</head>
<body class="bg" style="margin:0;padding:0;background:#f3f3f0;">
<div style="display:none;max-height:0;overflow:hidden;opacity:0;color:transparent;">{preheader}</div>
<table role="presentation" class="bg" width="100%" cellpadding="0" cellspacing="0" style="background:#f3f3f0;">
<tr><td align="center" style="padding:32px 16px;">
  <table role="presentation" class="card" width="100%" cellpadding="0" cellspacing="0" style="max-width:520px;background:#ffffff;border:1px solid #d8d8d2;border-radius:16px;padding:32px;">
    <tr><td style="padding:0 0 22px 0;">
      <table role="presentation" cellpadding="0" cellspacing="0"><tr>
        <td style="padding:0 12px 0 0;"><img src="{flame}" width="48" height="48" alt="" style="display:block;border:0;"></td>
        <td class="ink" style="font:700 20px/1 'IBM Plex Sans','Segoe UI',Helvetica,Arial,sans-serif;color:#121212;">Botloft</td>
      </tr></table>
    </td></tr>
    <tr><td class="ink" style="font:700 24px/1.3 'IBM Plex Sans','Segoe UI',Helvetica,Arial,sans-serif;color:#121212;padding:0 0 12px 0;">{title}</td></tr>
    <tr><td class="soft" style="font:400 16px/1.6 'IBM Plex Sans','Segoe UI',Helvetica,Arial,sans-serif;color:#3b3b38;padding:0 0 24px 0;">{intro}</td></tr>
    <tr><td style="padding:0 0 24px 0;">
      <table role="presentation" cellpadding="0" cellspacing="0"><tr>
        <td class="btn" bgcolor="{button}" style="background:{button};border-radius:10px;">
          <a href="{link}" style="display:inline-block;padding:14px 28px;font:600 16px/1 'IBM Plex Sans','Segoe UI',Helvetica,Arial,sans-serif;color:#ffffff;text-decoration:none;border-radius:10px;">{button_label}</a>
        </td>
      </tr></table>
    </td></tr>
    <tr><td class="muted" style="font:400 13px/1.5 'IBM Plex Sans','Segoe UI',Helvetica,Arial,sans-serif;color:#5f5f59;padding:0 0 8px 0;">{fallback}</td></tr>
    <tr><td style="padding:0 0 24px 0;">
      <div class="url" style="font:400 12px/1.5 'IBM Plex Mono',Consolas,'Courier New',monospace;color:#3b3b38;background:#f3f3f0;border:1px solid #d8d8d2;border-radius:8px;padding:10px 12px;word-break:break-all;"><a href="{link}" style="color:inherit;text-decoration:none;">{link}</a></div>
    </td></tr>
    <tr><td class="soft" style="font:400 14px/1.6 'IBM Plex Sans','Segoe UI',Helvetica,Arial,sans-serif;color:#3b3b38;padding:0 0 8px 0;">{note}</td></tr>
    <tr><td class="muted" style="font:400 13px/1.5 'IBM Plex Sans','Segoe UI',Helvetica,Arial,sans-serif;color:#5f5f59;padding:0;">{device_line}</td></tr>
  </table>
  <table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="max-width:520px;">
    <tr><td class="muted" align="center" style="font:400 12px/1.5 'IBM Plex Sans','Segoe UI',Helvetica,Arial,sans-serif;color:#5f5f59;padding:18px 8px 0 8px;">{footer}</td></tr>
  </table>
</td></tr>
</table>
</body>
</html>
"##,
        lang = escape(card.lang),
        title = escape(card.title),
        preheader = escape(card.preheader),
        flame = escape(card.flame_url),
        intro = escape(card.intro),
        button = button,
        button_dark = button_dark,
        button_ink_dark = button_ink_dark,
        button_label = escape(card.button),
        link = escape(card.link),
        fallback = escape(card.fallback),
        note = escape(card.note),
        device_line = escape(card.device_line),
        footer = escape(card.footer),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card() -> Card<'static> {
        Card {
            lang: "pt-BR",
            preheader: "Vale por 10 minutos.",
            title: "Entrar no Botloft",
            intro: "Aperte o botão em \"PC <da> Ana\".",
            button: "Entrar no Botloft",
            danger: false,
            link: "https://x.example/v1/login/confirm?code=abc&n=1",
            fallback: "Se o botão não abrir:",
            note: "Vale por 10 minutos.",
            device_line: "Pedido do computador: PC <da> Ana",
            footer: "Botloft",
            flame_url: "https://x.example/brand/flame.png",
        }
    }

    #[test]
    fn the_button_the_link_and_the_flame_are_there() {
        let html = render(&card());
        assert!(html.contains(r#"<a href="https://x.example/v1/login/confirm?code=abc&amp;n=1""#));
        assert!(html.contains(">Entrar no Botloft</a>"));
        assert!(html.contains(r#"src="https://x.example/brand/flame.png""#));
        assert!(html.contains(r#"<html lang="pt-BR">"#));
        // The link also sits as text, for a client that does not draw the button.
        assert!(html.contains(">https://x.example/v1/login/confirm?code=abc&amp;n=1</a></div>"));
        assert!(html.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn what_a_person_typed_cannot_become_markup_in_the_mail() {
        let html = render(&card());
        assert!(html.contains("PC &lt;da&gt; Ana"));
        assert!(!html.contains("<da>"));
    }

    #[test]
    fn an_irreversible_button_is_red() {
        let danger = render(&Card {
            danger: true,
            ..card()
        });
        assert!(danger.contains(r##"bgcolor="#bd3529""##));
        assert!(render(&card()).contains(r##"bgcolor="#b83a14""##));
    }
}
