use crate::ClientResult;

use super::{EmailSender, types::Email};

impl EmailSender {
    /// Sends one email. An invalid address is an error before connecting; SMTP refusals and
    /// connection problems come back as [`ClientError::Smtp`](crate::errors::ClientError::Smtp).
    pub async fn send_email(&self, email: &Email) -> ClientResult<()> {
        let message = self.build(email)?;
        self.deliver(message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::email::{
        config::{EmailConfig, EmailServer},
        types::BodyType,
    };

    #[tokio::test]
    async fn message_carries_sender_receiver_and_body_type() {
        let sender = EmailConfig::new(EmailServer::Gmail, "me@gmail.com", "Tready", "pw")
            .to_sender()
            .expect("sender");
        let email = Email::new("you@example.com", "제목", "<b>본문</b>", BodyType::Html)
            .receiver_name("홍길동");
        let message =
            String::from_utf8(sender.build(&email).expect("message").formatted()).expect("utf-8");
        assert!(message.contains("From: Tready <me@gmail.com>"));
        assert!(message.contains("you@example.com"));
        assert!(message.contains("Content-Type: text/html"));

        let bad = Email::new("not-an-address", "s", "b", BodyType::Text);
        assert!(sender.build(&bad).is_err());
    }
}
