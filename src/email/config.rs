use std::str::FromStr;

use lettre::{message::Mailbox, transport::smtp::authentication::Credentials};

use crate::{ClientResult, email::EmailSender, env, errors::ClientError};

/// SMTP servers the crate knows how to reach (STARTTLS on port 587).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailServer {
    Naver,
    Gmail,
}

impl EmailServer {
    pub(super) fn host(self) -> &'static str {
        match self {
            Self::Naver => "smtp.naver.com",
            Self::Gmail => "smtp.gmail.com",
        }
    }
}

impl FromStr for EmailServer {
    type Err = ClientError;

    /// `naver` or `gmail`, any case.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_lowercase().as_str() {
            "naver" => Ok(Self::Naver),
            "gmail" => Ok(Self::Gmail),
            _ => Err(ClientError::InvalidData("unknown email server")),
        }
    }
}

/// SMTP account. The username is also the sender address; the password is never shown by `Debug`.
pub struct EmailConfig {
    server: EmailServer,
    username: String,
    sender_name: String,
    password: String,
}

impl EmailConfig {
    pub fn new(
        server: EmailServer,
        username: impl Into<String>,
        sender_name: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            server,
            username: username.into(),
            sender_name: sender_name.into(),
            password: password.into(),
        }
    }

    /// `EMAIL_SERVER` (`naver` / `gmail`), `EMAIL_USERNAME`, `EMAIL_SENDER_NAME`, `EMAIL_PASSWORD`.
    pub fn from_env() -> ClientResult<Self> {
        Ok(Self::new(
            env::required("EMAIL_SERVER")?.parse()?,
            env::required("EMAIL_USERNAME")?,
            env::required("EMAIL_SENDER_NAME")?,
            env::required("EMAIL_PASSWORD")?,
        ))
    }

    pub fn to_sender(self) -> ClientResult<EmailSender> {
        EmailSender::new(self)
    }

    pub(super) fn server(&self) -> EmailServer {
        self.server
    }

    pub(super) fn credentials(&self) -> Credentials {
        Credentials::new(self.username.clone(), self.password.clone())
    }

    /// `"Sender Name" <username>`.
    pub(super) fn sender(&self) -> ClientResult<Mailbox> {
        let address = self
            .username
            .parse()
            .map_err(|_| ClientError::InvalidData("EMAIL_USERNAME is not an email address"))?;
        Ok(Mailbox::new(Some(self.sender_name.clone()), address))
    }
}

impl std::fmt::Debug for EmailConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmailConfig")
            .field("server", &self.server)
            .field("username", &self.username)
            .field("sender_name", &self.sender_name)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn servers_parse_and_resolve_hosts() {
        assert_eq!("Naver".parse::<EmailServer>().unwrap(), EmailServer::Naver);
        assert_eq!(EmailServer::Gmail.host(), "smtp.gmail.com");
        assert!("daum".parse::<EmailServer>().is_err());
    }

    #[test]
    fn sender_needs_an_address_and_debug_hides_the_password() {
        let config = EmailConfig::new(EmailServer::Naver, "me@naver.com", "Tready", "secret");
        assert_eq!(
            config.sender().unwrap().to_string(),
            "Tready <me@naver.com>"
        );
        assert!(!format!("{config:?}").contains("secret"));
        let bad = EmailConfig::new(EmailServer::Naver, "me", "Tready", "secret");
        assert!(bad.sender().is_err());
    }
}
