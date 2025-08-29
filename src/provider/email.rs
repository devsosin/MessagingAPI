use lettre::{
    Transport,
    message::{Mailbox, MessageBuilder, header::ContentType},
};

use crate::{EmailSender, errors::ClientError, types::ClientResult};

impl EmailSender {
    pub(crate) async fn send(
        &self,
        receiver_mailbox: &Mailbox,
        subject: &str,
        is_html: bool,
        content: &str,
    ) -> ClientResult<()> {
        let message_builder = Into::<MessageBuilder>::into(&self.config);

        let content_type: ContentType = match is_html {
            true => ContentType::TEXT_HTML,
            false => ContentType::TEXT_PLAIN,
        };

        let message = message_builder
            .to(receiver_mailbox.to_owned())
            .subject(subject)
            .header(content_type)
            .body(String::from(content))
            .unwrap();

        match self.mailer.send(&message) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("{:?}", e);

                Err(ClientError::EmailError)
            }
        }
    }
}
