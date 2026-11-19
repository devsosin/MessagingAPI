use crate::{
    ClientResult,
    errors::ClientError,
    types::{Failure, Outcome},
};

use super::{
    Solapi,
    types::{
        Alimtalk, Balance, MAX_MESSAGES, MessageListResponse, MessageStatus, SendManyBody,
        SendReceipt, SendableTemplateQuery, Template,
    },
};

impl Solapi {
    /// Sends alimtalks in one group (`messages/v4/send-many/detail`). `Sent` means accepted: Kakao delivery
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

    /// Messages of a send group with their status codes (`messages/v4/list`), Solapi's fallback
    /// texts included. Read them with [`Delivery::evaluate`](super::types::Delivery::evaluate).
    pub async fn group_messages(&self, group_id: &str) -> Result<Vec<MessageStatus>, Failure> {
        // Group ids are alphanumeric; anything else must not reach the query string.
        let group_id: String = group_id
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .collect();
        let list: MessageListResponse = self
            .get(&format!("messages/v4/list?groupId={group_id}"))
            .await?;
        Ok(list.into_messages())
    }

    /// Account balance (`cash/v1/balance`). No side effect, so it also serves as a connection test.
    pub async fn balance(&self) -> Result<Balance, Failure> {
        self.get("cash/v1/balance").await
    }

    /// Templates that can be sent now (`kakao/v2/templates/sendable`): approved, not hidden or
    /// deleted. No pagination. Use it to see a template's variables before sending.
    pub async fn sendable_templates(
        &self,
        query: &SendableTemplateQuery,
    ) -> Result<Vec<Template>, Failure> {
        self.get(&format!(
            "kakao/v2/templates/sendable{}",
            query.to_query_string()
        ))
        .await
    }

    /// One template (`kakao/v2/templates/:templateId`). Returns that template's current status.
    pub async fn template(&self, template_id: &str) -> Result<Template, Failure> {
        // Template ids are alphanumeric (with `-` and `_`); anything else must not reach the path.
        let template_id: String = template_id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
            .collect();
        self.get(&format!("kakao/v2/templates/{template_id}")).await
    }
}
