use lettre::{AsyncTransport, Message, message::Mailbox};

use crate::{ClientResult, errors::ClientError};

use super::{EmailSender, types::Email};

impl EmailSender {
    pub(super) fn build(&self, email: &Email) -> ClientResult<Message> {
        let (name, address) = email.receiver();
        let address = address
            .parse()
            .map_err(|_| ClientError::InvalidData("invalid receiver email"))?;
        Ok(Message::builder()
            .from(self.sender.clone())
            .to(Mailbox::new(name.map(str::to_owned), address))
            .subject(email.subject())
            .header(email.body_type().content_type())
            .body(email.body().to_owned())?)
    }

    pub(super) async fn deliver(&self, message: Message) -> ClientResult<()> {
        self.mailer.send(message).await?;
        Ok(())
    }
}
