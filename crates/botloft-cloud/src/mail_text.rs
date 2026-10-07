//! The words of the e-mails, in the three languages of the app (spec 27.2,
//! 15.6): each one as text and as HTML (`mail_html.rs`).

use crate::mail_html::{self, Card};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    En,
    PtBr,
    Es,
}

impl Locale {
    /// The tag the app sends (`en`, `pt-BR`, `es`); anything else is English.
    pub fn parse(tag: &str) -> Self {
        Self::from_tag(tag).unwrap_or(Self::En)
    }

    fn from_tag(tag: &str) -> Option<Self> {
        let tag = tag.trim().to_ascii_lowercase();
        match tag.split(['-', '_']).next()? {
            "pt" => Some(Self::PtBr),
            "es" => Some(Self::Es),
            "en" => Some(Self::En),
            _ => None,
        }
    }

    /// The first language a browser lists that we speak.
    pub fn from_accept_language(header: &str) -> Self {
        header
            .split(',')
            .filter_map(|part| Self::from_tag(part.split(';').next().unwrap_or("")))
            .next()
            .unwrap_or(Self::En)
    }

    /// The language the browser asked for.
    pub fn of_request(headers: &axum::http::HeaderMap) -> Self {
        headers
            .get(axum::http::header::ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok())
            .map_or(Self::En, Self::from_accept_language)
    }

    /// The tag to store and send back.
    pub fn tag(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::PtBr => "pt-BR",
            Self::Es => "es",
        }
    }
}

/// An e-mail ready to send: what the inbox lists, and its two bodies.
pub struct Letter {
    pub subject: String,
    pub text: String,
    pub html: String,
}

/// What one kind of e-mail says in one language. `{device}` is where the name
/// of the computer goes.
struct Words {
    subject: &'static str,
    preheader: &'static str,
    title: &'static str,
    intro: &'static str,
    button: &'static str,
    note: &'static str,
    /// The first line of the text version, before the link.
    lead: &'static str,
    /// The line of the text version about what to do when it was not them.
    text_note: &'static str,
}

struct Common {
    fallback: &'static str,
    device_line: &'static str,
    footer: &'static str,
}

fn common(locale: Locale) -> Common {
    match locale {
        Locale::En => Common {
            fallback: "If the button does not open, copy this address into your browser:",
            device_line: "Asked from the computer \"{device}\".",
            footer: "You got this e-mail because someone asked to use Botloft with this address.",
        },
        Locale::PtBr => Common {
            fallback: "Se o botão não abrir, copie este endereço no navegador:",
            device_line: "Pedido feito do computador \"{device}\".",
            footer: "Você recebeu este e-mail porque alguém pediu para usar o Botloft com este endereço.",
        },
        Locale::Es => Common {
            fallback: "Si el botón no se abre, copia esta dirección en el navegador:",
            device_line: "Pedido desde el equipo \"{device}\".",
            footer: "Recibiste este correo porque alguien pidió usar Botloft con esta dirección.",
        },
    }
}

fn sign_in_words(locale: Locale) -> Words {
    match locale {
        Locale::En => Words {
            subject: "Your Botloft sign-in link",
            preheader: "Press the button to sign in. It works for 10 minutes.",
            title: "Sign in to Botloft",
            intro: "You asked to sign in to Botloft on \"{device}\". Press the button to confirm.",
            button: "Sign in to Botloft",
            note: "The link works once and expires in 10 minutes. If you did not ask for it, ignore this e-mail: nothing happens.",
            lead: "Open this link to sign in to Botloft on \"{device}\":",
            text_note: "It works once and expires in 10 minutes. If you did not ask for it, ignore this e-mail: nothing happens.",
        },
        Locale::PtBr => Words {
            subject: "Seu link de entrada no Botloft",
            preheader: "Aperte o botão para entrar. Ele vale por 10 minutos.",
            title: "Entrar no Botloft",
            intro: "Você pediu para entrar no Botloft em \"{device}\". Aperte o botão para confirmar.",
            button: "Entrar no Botloft",
            note: "O link vale uma vez e expira em 10 minutos. Se não foi você que pediu, ignore este e-mail: nada acontece.",
            lead: "Abra este link para entrar no Botloft em \"{device}\":",
            text_note: "Ele vale uma vez e expira em 10 minutos. Se não foi você que pediu, ignore este e-mail: nada acontece.",
        },
        Locale::Es => Words {
            subject: "Tu enlace para entrar en Botloft",
            preheader: "Pulsa el botón para entrar. Vale 10 minutos.",
            title: "Entrar en Botloft",
            intro: "Pediste entrar en Botloft en \"{device}\". Pulsa el botón para confirmar.",
            button: "Entrar en Botloft",
            note: "El enlace sirve una vez y caduca en 10 minutos. Si no lo pediste tú, ignora este correo: no pasa nada.",
            lead: "Abre este enlace para entrar en Botloft en \"{device}\":",
            text_note: "Sirve una vez y caduca en 10 minutos. Si no lo pediste tú, ignora este correo: no pasa nada.",
        },
    }
}

fn delete_words(locale: Locale) -> Words {
    match locale {
        Locale::En => Words {
            subject: "Delete your Botloft account?",
            preheader: "This deletes your account and every copy saved in it.",
            title: "Delete your Botloft account?",
            intro: "You asked to delete your account and every copy saved in it. This cannot be undone.",
            button: "Delete my account",
            note: "The link expires in 10 minutes. If you did not ask for this, ignore this e-mail: nothing is deleted.",
            lead: "Open this link and press the button to delete your Botloft account and every copy saved in it:",
            text_note: "It expires in 10 minutes. If you did not ask for this, ignore this e-mail: nothing is deleted.",
        },
        Locale::PtBr => Words {
            subject: "Apagar sua conta do Botloft?",
            preheader: "Isto apaga a sua conta e todas as cópias guardadas nela.",
            title: "Apagar sua conta do Botloft?",
            intro: "Você pediu para apagar a sua conta e todas as cópias guardadas nela. Não dá para desfazer.",
            button: "Apagar minha conta",
            note: "O link expira em 10 minutos. Se não foi você que pediu, ignore este e-mail: nada é apagado.",
            lead: "Abra este link e aperte o botão para apagar sua conta do Botloft e todas as cópias guardadas nela:",
            text_note: "Ele expira em 10 minutos. Se não foi você que pediu, ignore este e-mail: nada é apagado.",
        },
        Locale::Es => Words {
            subject: "¿Borrar tu cuenta de Botloft?",
            preheader: "Esto borra tu cuenta y todas las copias guardadas en ella.",
            title: "¿Borrar tu cuenta de Botloft?",
            intro: "Pediste borrar tu cuenta y todas las copias guardadas en ella. No se puede deshacer.",
            button: "Borrar mi cuenta",
            note: "El enlace caduca en 10 minutos. Si no lo pediste tú, ignora este correo: no se borra nada.",
            lead: "Abre este enlace y pulsa el botón para borrar tu cuenta de Botloft y todas las copias guardadas en ella:",
            text_note: "Caduca en 10 minutos. Si no lo pediste tú, ignora este correo: no se borra nada.",
        },
    }
}

fn letter(
    locale: Locale,
    words: &Words,
    link: &str,
    device: Option<&str>,
    brand_base: &str,
    danger: bool,
) -> Letter {
    let common = common(locale);
    let name = device.unwrap_or("");
    let fill = |text: &str| text.replace("{device}", name);
    let device_line = if device.is_some() {
        fill(common.device_line)
    } else {
        String::new()
    };
    let mut text = format!("{}\n\n{link}\n\n{}\n", fill(words.lead), words.text_note);
    if !device_line.is_empty() {
        text.push_str(&format!("\n{device_line}\n"));
    }
    let html = mail_html::render(&Card {
        lang: locale.tag(),
        preheader: words.preheader,
        title: words.title,
        intro: &fill(words.intro),
        button: words.button,
        danger,
        link,
        fallback: common.fallback,
        note: words.note,
        device_line: &device_line,
        footer: common.footer,
        flame_url: &format!("{}/brand/flame.png", brand_base.trim_end_matches('/')),
    });
    Letter {
        subject: words.subject.to_owned(),
        text,
        html,
    }
}

/// The e-mail with the sign-in link; `brand_base` is the server's address.
pub fn login_mail(locale: Locale, link: &str, device: &str, brand_base: &str) -> Letter {
    letter(
        locale,
        &sign_in_words(locale),
        link,
        Some(device),
        brand_base,
        false,
    )
}

/// The e-mail that confirms deleting an account.
pub fn delete_mail(locale: Locale, link: &str, brand_base: &str) -> Letter {
    letter(locale, &delete_words(locale), link, None, brand_base, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://cloud.example.org/";

    #[test]
    fn tags_and_browser_headers_pick_a_language() {
        assert_eq!(Locale::parse("pt-BR"), Locale::PtBr);
        assert_eq!(Locale::parse("es_MX"), Locale::Es);
        assert_eq!(Locale::parse("fr"), Locale::En);
        assert_eq!(
            Locale::from_accept_language("fr, es;q=0.8, en;q=0.5"),
            Locale::Es
        );
        assert_eq!(Locale::from_accept_language(""), Locale::En);
    }

    #[test]
    fn every_language_carries_the_link_the_device_and_both_bodies() {
        for locale in [Locale::En, Locale::PtBr, Locale::Es] {
            let letter = login_mail(locale, "https://x/y?code=1&n=2", "Casa <PC>", BASE);
            assert!(!letter.subject.is_empty());
            assert!(
                letter.text.contains("https://x/y?code=1&n=2") && letter.text.contains("Casa <PC>")
            );
            assert!(letter.html.contains("https://x/y?code=1&amp;n=2"));
            assert!(letter.html.contains("Casa &lt;PC&gt;") && !letter.html.contains("Casa <PC>"));
            assert!(
                letter
                    .html
                    .contains("https://cloud.example.org/brand/flame.png")
            );
            assert!(
                letter
                    .html
                    .contains(&format!("<html lang=\"{}\">", locale.tag()))
            );
            assert!(!letter.html.contains("{device}") && !letter.text.contains("{device}"));
        }
    }

    #[test]
    fn deleting_an_account_asks_in_red_and_names_no_device() {
        for locale in [Locale::En, Locale::PtBr, Locale::Es] {
            let letter = delete_mail(locale, "https://x/del?code=2", BASE);
            assert!(letter.html.contains(r##"bgcolor="#bd3529""##));
            assert!(letter.text.contains("https://x/del?code=2"));
            assert!(!letter.text.contains("{device}") && !letter.html.contains("{device}"));
        }
        // The words the app and its tests rely on.
        assert!(
            login_mail(Locale::PtBr, "l", "d", BASE)
                .text
                .contains("Abra este link")
        );
        assert!(
            login_mail(Locale::PtBr, "l", "d", BASE)
                .text
                .contains("expira em 10 minutos")
        );
    }
}
