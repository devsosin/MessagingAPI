use crate::{
    ClientResult,
    types::{Failure, Outcome},
};

use super::{
    AligoAPI,
    types::{Remain, SendReceipt, TextMessage, TextType, image_fields, mass_fields},
};

impl AligoAPI {
    /// Sends up to 500 texts in one request (`send_mass/`). Invalid input is an error before
    /// anything is sent; Aligo's answer is the [`Outcome`].
    pub async fn send_mass(
        &self,
        messages: &[TextMessage],
        text_type: TextType,
    ) -> ClientResult<Outcome<SendReceipt>> {
        let fields = mass_fields(messages, text_type)?;
        Ok(self.send_form("send_mass/", fields).await)
    }

    /// Sends one image message (`send/`, MMS) with a JPEG attached.
    pub async fn send_image(
        &self,
        receiver: &str,
        message: &str,
        jpeg: Vec<u8>,
    ) -> ClientResult<Outcome<SendReceipt>> {
        self.send_with_image("send/", image_fields(receiver, message), jpeg)
            .await
    }

    /// Remaining sendable counts (`remain/`). No side effect, so it also serves as a connection test.
    pub async fn remain(&self) -> Result<Remain, Failure> {
        self.fetch_remain().await
    }
}
