use crate::{
    ClientResult,
    errors::ClientError,
    types::{Failure, Outcome},
};

use super::{
    Solapi,
    types::{Alimtalk, Balance, MAX_MESSAGES, SendManyBody, SendReceipt},
};

impl Solapi {
    /// Sends alimtalks in one group (`messages/v4/send-many`). `Sent` means accepted: Kakao delivery
    /// (or Solapi's text fallback) is known only later from the group's message list.
    pub async fn send_alimtalks(
        &self,
        messages: &[Alimtalk],
    ) -> ClientResult<Outcome<SendReceipt>> {
        if messages.is_empty() {
            return Err(ClientError::InvalidData("no messages to send"));
        }
        if messages.len() > MAX_MESSAGES {
            return Err(ClientError::InvalidData(
                "send-many takes at most 10,000 messages",
            ));
        }
        let body = SendManyBody::new(messages, self.config.sender(), self.config.pf_id());
        Ok(self.post_send_many(&body).await)
    }

    /// Account balance (`cash/v1/balance`). No side effect, so it also serves as a connection test.
    pub async fn balance(&self) -> Result<Balance, Failure> {
        self.get("cash/v1/balance").await
    }
}
