use std::env;

use lettre::{
    Message,
    message::{Mailbox, MessageBuilder},
    transport::smtp::authentication::Credentials,
};

use crate::email::EmailSender;

pub(crate) enum EmailServer {
    Naver,
    Gmail,
}

impl From<&str> for EmailServer {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "naver" => Self::Naver,
            "gmail" => Self::Gmail,
            _ => panic!("Unknown Email Server"),
        }
    }
}

impl Into<&str> for &EmailServer {
    fn into(self) -> &'static str {
        match &self {
            EmailServer::Naver => "naver",
            EmailServer::Gmail => "gmail",
        }
    }
}

pub struct EmailConfig {
    email_server: EmailServer,
    email_username: String,
    email_sender_name: String,
    email_password: String,
}

impl Into<Credentials> for &EmailConfig {
    fn into(self) -> Credentials {
        Credentials::new(
            self.email_username.to_owned(),
            self.email_password.to_owned(),
        )
    }
}

impl Into<MessageBuilder> for &EmailConfig {
    fn into(self) -> MessageBuilder {
        Message::builder().from(Mailbox::new(
            Some(self.email_sender_name.to_owned()),
            self.email_username.parse().unwrap(),
        ))
    }
}

impl EmailConfig {
    pub fn from_env() -> Self {
        let email_server = env::var("EMAIL_SERVER").expect("Failed to get env variable");
        let email_username = env::var("EMAIL_USERNAME").expect("Failed to get env variable");
        let email_sender_name = env::var("EMAIL_SENDER_NAME").expect("Failed to get env variable");
        let email_password = env::var("EMAIL_PASSWORD").expect("Failed to get env variable");

        Self {
            email_server: email_server.as_str().into(),
            email_username,
            email_sender_name,
            email_password,
        }
    }

    pub(crate) fn get_server(&self) -> &EmailServer {
        &self.email_server
    }

    pub fn to_sender(self) -> EmailSender {
        EmailSender::new(self)
    }
}
