use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::types::{Failure, FailureKind, Outcome};

/// Most messages one `send-many` request takes.
pub const MAX_MESSAGES: usize = 10_000;

/// One Kakao alimtalk. Variables are template names (`회원명`) or already wrapped (`#{회원명}`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alimtalk {
    to: String,
    template_id: String,
    variables: BTreeMap<String, String>,
}

impl Alimtalk {
    pub fn new<K, V>(
        to: impl Into<String>,
        template_id: impl Into<String>,
        variables: impl IntoIterator<Item = (K, V)>,
    ) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            to: to.into(),
            template_id: template_id.into(),
            variables: variables
                .into_iter()
                .map(|(k, v)| (template_variable(k.into()), v.into()))
                .collect(),
        }
    }
}

fn template_variable(name: String) -> String {
    if name.starts_with("#{") && name.ends_with('}') {
        name
    } else {
        format!("#{{{name}}}")
    }
}

/// `messages/v4/send-many` body.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SendManyBody<'a> {
    allow_duplicates: bool,
    messages: Vec<WireMessage<'a>>,
}

impl<'a> SendManyBody<'a> {
    pub(super) fn new(messages: &'a [Alimtalk], from: &'a str, pf_id: &'a str) -> Self {
        Self {
            allow_duplicates: true,
            messages: messages
                .iter()
                .map(|m| WireMessage {
                    to: m.to.as_str(),
                    from,
                    kakao_options: KakaoOptions {
                        pf_id,
                        template_id: m.template_id.as_str(),
                        variables: &m.variables,
                    },
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WireMessage<'a> {
    to: &'a str,
    from: &'a str,
    kakao_options: KakaoOptions<'a>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct KakaoOptions<'a> {
    pf_id: &'a str,
    template_id: &'a str,
    variables: &'a BTreeMap<String, String>,
}

/// Status of one message as Solapi reports it (`statusCode` with `statusMessage` or `reason`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageStatus {
    #[serde(default)]
    message_id: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default, rename = "type")]
    message_type: Option<String>,
    status_code: String,
    #[serde(default, alias = "reason")]
    status_message: String,
    #[serde(default)]
    replacement: bool,
}

impl MessageStatus {
    pub fn message_id(&self) -> Option<&str> {
        self.message_id.as_deref()
    }

    pub fn to(&self) -> Option<&str> {
        self.to.as_deref()
    }

    /// `ATA` (alimtalk), `SMS`, `LMS`, …
    pub fn message_type(&self) -> Option<&str> {
        self.message_type.as_deref()
    }

    pub fn status_code(&self) -> &str {
        &self.status_code
    }

    pub fn status_message(&self) -> &str {
        &self.status_message
    }

    /// `true` for the text Solapi sent in place of a failed Kakao message.
    pub fn is_replacement(&self) -> bool {
        self.replacement
    }

    /// `"3107 72시간 이내 카카오톡 미사용자"`
    pub fn describe(&self) -> String {
        format!("{} {}", self.status_code, self.status_message)
            .trim()
            .to_owned()
    }
}

/// What Solapi accepted: the group to look up later and the messages refused at registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendReceipt {
    group_id: String,
    registered: i64,
    refused: Vec<MessageStatus>,
}

impl SendReceipt {
    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    pub fn registered(&self) -> i64 {
        self.registered
    }

    /// Messages refused at registration; the others are in the group.
    pub fn refused(&self) -> &[MessageStatus] {
        &self.refused
    }
}

/// A 2xx `send-many` answer: the group at the top level, or under `groupInfo` (`send-many/detail`).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SendManyResponse {
    #[serde(flatten)]
    group: GroupInfo,
    #[serde(default)]
    group_info: Option<GroupInfo>,
    #[serde(default)]
    failed_message_list: Vec<MessageStatus>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GroupInfo {
    #[serde(default, alias = "_id")]
    group_id: Option<String>,
    #[serde(default)]
    count: Count,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Count {
    #[serde(default)]
    registered_failed: i64,
    #[serde(default)]
    registered_success: i64,
}

impl SendManyResponse {
    /// Every message refused at registration → nothing was sent and no result will follow.
    pub(super) fn into_outcome(self) -> Outcome<SendReceipt> {
        let group = self.group_info.unwrap_or(self.group);
        if group.count.registered_success == 0 && group.count.registered_failed > 0 {
            let detail = self
                .failed_message_list
                .first()
                .map(MessageStatus::describe)
                .unwrap_or_else(|| "every message was refused at registration".into());
            return Outcome::Failed(Failure::from_message(&detail));
        }
        match group.group_id {
            Some(group_id) => Outcome::Sent(SendReceipt {
                group_id,
                registered: group.count.registered_success,
                refused: self.failed_message_list,
            }),
            // Accepted, but nothing to look the result up with.
            None => Outcome::Unknown,
        }
    }
}

/// Error body: `{ "errorCode": "…", "errorMessage": "…" }`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ErrorResponse {
    #[serde(default)]
    error_code: String,
    #[serde(default)]
    error_message: String,
}

impl ErrorResponse {
    pub(super) fn into_failure(self, status: u16) -> Failure {
        let detail = format!("{} {}", self.error_code, self.error_message)
            .trim()
            .to_owned();
        let kind = match status {
            401 | 403 if detail.to_lowercase().contains("ip") => FailureKind::IpNotAllowed,
            401 | 403 => FailureKind::AuthFailed,
            _ => FailureKind::from_message(&detail),
        };
        Failure::new(kind, detail)
    }
}

/// Account balance (`cash/v1/balance`).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Balance {
    #[serde(default)]
    balance: f64,
    #[serde(default)]
    point: f64,
}

impl Balance {
    pub fn balance(&self) -> f64 {
        self.balance
    }

    pub fn point(&self) -> f64 {
        self.point
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn send_answer(value: serde_json::Value) -> Outcome<SendReceipt> {
        serde_json::from_value::<SendManyResponse>(value)
            .expect("send answer")
            .into_outcome()
    }

    #[test]
    fn body_wraps_variables_and_fills_sender() {
        let talks = [Alimtalk::new(
            "01000000000",
            "TPL",
            [("회원명", "홍길동"), ("#{코드}", "A-1")],
        )];
        let body = serde_json::to_value(SendManyBody::new(&talks, "0200000000", "PF")).unwrap();
        assert_eq!(
            body,
            json!({
                "allowDuplicates": true,
                "messages": [{
                    "to": "01000000000",
                    "from": "0200000000",
                    "kakaoOptions": {
                        "pfId": "PF",
                        "templateId": "TPL",
                        "variables": { "#{회원명}": "홍길동", "#{코드}": "A-1" }
                    }
                }]
            })
        );
    }

    #[test]
    fn send_answer_checks_registration() {
        // `send-many`: the group at the top level.
        let Outcome::Sent(receipt) = send_answer(json!({
            "groupId": "G1", "count": { "registeredFailed": 0, "registeredSuccess": 1 }
        })) else {
            panic!("expected sent")
        };
        assert_eq!(receipt.group_id(), "G1");

        // `send-many/detail` (Solapi's example): one refused, one registered → still sent.
        let Outcome::Sent(partly) = send_answer(json!({
            "groupInfo": { "groupId": "G2", "count": { "registeredFailed": 1, "registeredSuccess": 1 } },
            "failedMessageList": [{ "to": "01099999999", "type": "SMS", "statusCode": "3040",
                                    "statusMessage": "등록되지 않은 발신번호입니다." }],
            "messageList": [{ "messageId": "M1", "statusCode": "2000", "statusMessage": "정상 접수" }]
        })) else {
            panic!("expected sent")
        };
        assert_eq!(partly.group_id(), "G2");
        assert_eq!(
            partly.refused()[0].describe(),
            "3040 등록되지 않은 발신번호입니다."
        );

        // Every message refused: failed now, nothing to wait for.
        let Outcome::Failed(failure) = send_answer(json!({
            "groupInfo": { "groupId": "G3", "count": { "registeredFailed": 1, "registeredSuccess": 0 } },
            "failedMessageList": [{ "statusCode": "1030", "statusMessage": "잔액 부족" }]
        })) else {
            panic!("expected failed")
        };
        assert_eq!(failure.kind(), FailureKind::LowBalance);
        assert_eq!(failure.detail(), "1030 잔액 부족");
    }

    #[test]
    fn error_answers_map_to_failures() {
        let error = |status, body: serde_json::Value| {
            serde_json::from_value::<ErrorResponse>(body)
                .unwrap()
                .into_failure(status)
        };
        let ip = error(
            403,
            json!({ "errorCode": "Forbidden", "errorMessage": "허용되지 않은 IP" }),
        );
        assert_eq!(ip.kind(), FailureKind::IpNotAllowed);
        let auth = error(
            401,
            json!({ "errorCode": "InvalidApiKey", "errorMessage": "" }),
        );
        assert_eq!(auth.kind(), FailureKind::AuthFailed);
        let money = error(
            400,
            json!({ "errorCode": "NotEnoughBalance", "errorMessage": "잔액이 부족합니다" }),
        );
        assert_eq!(money.kind(), FailureKind::LowBalance);
    }
}
