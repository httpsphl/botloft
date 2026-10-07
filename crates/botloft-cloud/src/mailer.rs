//! Sending e-mail goes through here, so tests read an outbox instead of
//! talking to a mail server (spec 27.2).

use std::sync::{Arc, Mutex};

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::config::{self, Smtp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mail {
    pub to: String,
    pub subject: String,
    pub body: String,
}

/// Mail kept in memory, for tests.
#[derive(Clone, Default)]
pub struct Outbox {
    mails: Arc<Mutex<Vec<Mail>>>,
    broken: bool,
}

impl Outbox {
    /// An outbox whose mail never goes, to see what the server does then.
    pub fn broken() -> Self {
        Self {
            broken: true,
            ..Self::default()
        }
    }

    pub fn sent(&self) -> Vec<Mail> {
        self.mails
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

pub struct SmtpMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
}

impl SmtpMailer {
    pub fn new(smtp: &Smtp) -> anyhow::Result<Self> {
        let password = config::secret(
            smtp.password_file.as_deref(),
            config::SMTP_PASSWORD_VAR,
            "smtp password",
        )?;
        let builder = if smtp.port == 465 {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp.host)?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.host)?
        };
        let transport = builder
            .port(smtp.port)
            .credentials(Credentials::new(smtp.user.clone(), password))
            .build();
        Ok(Self {
            transport,
            from: smtp.from.clone(),
        })
    }

    async fn send(&self, mail: &Mail) -> anyhow::Result<()> {
        let message = Message::builder()
            .from(self.from.parse()?)
            .to(mail.to.parse()?)
            .subject(mail.subject.as_str())
            .header(ContentType::TEXT_PLAIN)
            .body(mail.body.clone())?;
        self.transport.send(message).await?;
        Ok(())
    }
}

/// The mail did not go. Nothing more: a mail server may repeat the address.
#[derive(Debug)]
pub struct SendFailed;

pub enum Mailer {
    Smtp(Box<SmtpMailer>),
    Outbox(Outbox),
}

impl Mailer {
    pub async fn send(&self, mail: Mail) -> Result<(), SendFailed> {
        match self {
            Self::Outbox(outbox) if outbox.broken => Err(SendFailed),
            Self::Outbox(outbox) => {
                outbox
                    .mails
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .push(mail);
                Ok(())
            }
            Self::Smtp(smtp) => smtp.send(&mail).await.map_err(|_| {
                // Nothing of the error: a mail server may repeat the address.
                tracing::error!("sending an e-mail failed");
                SendFailed
            }),
        }
    }
}
