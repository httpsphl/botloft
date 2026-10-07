//! The words of the sign-in e-mail, in the three languages of the app
//! (spec 27.2, 15.6).

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

/// The subject and the text of the e-mail with the sign-in link.
pub fn login_mail(locale: Locale, link: &str, device: &str) -> (String, String) {
    match locale {
        Locale::En => (
            "Your Botloft sign-in link".to_owned(),
            format!(
                "Open this link to sign in to Botloft on \"{device}\":\n\n{link}\n\n\
                 It works once and expires in 10 minutes. If you did not ask for it, \
                 ignore this e-mail: nothing happens.\n"
            ),
        ),
        Locale::PtBr => (
            "Seu link de entrada no Botloft".to_owned(),
            format!(
                "Abra este link para entrar no Botloft em \"{device}\":\n\n{link}\n\n\
                 Ele vale uma vez e expira em 10 minutos. Se não foi você que pediu, \
                 ignore este e-mail: nada acontece.\n"
            ),
        ),
        Locale::Es => (
            "Tu enlace para entrar en Botloft".to_owned(),
            format!(
                "Abre este enlace para entrar en Botloft en \"{device}\":\n\n{link}\n\n\
                 Sirve una vez y caduca en 10 minutos. Si no lo pediste tú, \
                 ignora este correo: no pasa nada.\n"
            ),
        ),
    }
}

/// The subject and the text of the e-mail that confirms deleting an account.
pub fn delete_mail(locale: Locale, link: &str) -> (String, String) {
    match locale {
        Locale::En => (
            "Delete your Botloft account?".to_owned(),
            format!(
                "Open this link and press the button to delete your Botloft account and every                  copy saved in it:

{link}

It expires in 10 minutes. If you did not ask                  for this, ignore this e-mail: nothing is deleted.
"
            ),
        ),
        Locale::PtBr => (
            "Apagar sua conta do Botloft?".to_owned(),
            format!(
                "Abra este link e aperte o botão para apagar sua conta do Botloft e todas as                  cópias guardadas nela:

{link}

Ele expira em 10 minutos. Se não foi você                  que pediu, ignore este e-mail: nada é apagado.
"
            ),
        ),
        Locale::Es => (
            "¿Borrar tu cuenta de Botloft?".to_owned(),
            format!(
                "Abre este enlace y pulsa el botón para borrar tu cuenta de Botloft y todas las                  copias guardadas en ella:

{link}

Caduca en 10 minutos. Si no lo pediste                  tú, ignora este correo: no se borra nada.
"
            ),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn every_language_carries_the_link() {
        for locale in [Locale::En, Locale::PtBr, Locale::Es] {
            let (subject, body) = login_mail(locale, "https://x/y", "Casa");
            assert!(!subject.is_empty());
            assert!(body.contains("https://x/y") && body.contains("Casa"));
        }
    }
}
