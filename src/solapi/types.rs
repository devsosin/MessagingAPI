use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

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
    #[serde(deserialize_with = "code_text")]
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

    pub fn class(&self) -> StatusClass {
        StatusClass::of(&self.status_code)
    }
}

/// A status code may come as a string (`"4000"`) or a number.
fn code_text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Code {
        Text(String),
        Number(i64),
    }
    Ok(match Code::deserialize(deserializer)? {
        Code::Text(s) => s,
        Code::Number(n) => n.to_string(),
    })
}

/// Solapi status codes by class. The fine-grained codes are not consistent enough to rely on (the
/// official example pairs `3040` with "등록되지 않은 발신번호" while the code table says 전송시간 초과),
/// so decisions use the class only and keep the code and message for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    /// `2000` accepted, `3000` handed to the carrier and waiting for its report.
    InProgress,
    /// `4000`.
    Received,
    /// Every other code: `1xxx` at registration, `2xxx` in Solapi, `3xxx` at the carrier or Kakao.
    Failed,
}

impl StatusClass {
    pub fn of(code: &str) -> Self {
        match code {
            "2000" | "3000" => Self::InProgress,
            "4000" => Self::Received,
            _ => Self::Failed,
        }
    }
}

/// Whether a group reached its receivers, counting Solapi's text fallback for a failed Kakao message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivery {
    /// Some message (the Kakao one, or its fallback text) is still in progress.
    InProgress,
    /// Every Kakao message was received.
    Delivered,
    /// A Kakao message failed and the fallback text was received.
    DeliveredByFallback {
        failed: Vec<MessageStatus>,
        fallback: Vec<MessageStatus>,
    },
    /// A Kakao message failed and there is no fallback text, or not yet: Solapi may add it a moment
    /// after the failure, so the caller decides how long to wait before treating this as final.
    FailedWithoutFallback { failed: Vec<MessageStatus> },
    /// A Kakao message failed and its fallback text failed too.
    Failed {
        failed: Vec<MessageStatus>,
        fallback_failed: Vec<MessageStatus>,
    },
}

impl Delivery {
    /// Reads a group's messages (`messages/v4/list`).
    pub fn evaluate(messages: &[MessageStatus]) -> Self {
        let (fallback, originals): (Vec<&MessageStatus>, Vec<&MessageStatus>) =
            messages.iter().partition(|m| m.is_replacement());
        let in_progress =
            |list: &[&MessageStatus]| list.iter().any(|m| m.class() == StatusClass::InProgress);
        let failed_of = |list: &[&MessageStatus]| -> Vec<MessageStatus> {
            list.iter()
                .filter(|m| m.class() == StatusClass::Failed)
                .map(|m| (*m).clone())
                .collect()
        };

        if originals.is_empty() || in_progress(&originals) {
            return Self::InProgress;
        }
        let failed = failed_of(&originals);
        if failed.is_empty() {
            return Self::Delivered;
        }
        if fallback.is_empty() {
            return Self::FailedWithoutFallback { failed };
        }
        if in_progress(&fallback) {
            return Self::InProgress;
        }
        let fallback_failed = failed_of(&fallback);
        if fallback_failed.is_empty() {
            Self::DeliveredByFallback {
                failed,
                fallback: fallback.into_iter().cloned().collect(),
            }
        } else {
            Self::Failed {
                failed,
                fallback_failed,
            }
        }
    }
}

/// `messages/v4/list` answer: messages keyed by message id.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct MessageListResponse {
    #[serde(default)]
    message_list: BTreeMap<String, MessageStatus>,
}

impl MessageListResponse {
    pub(super) fn into_messages(self) -> Vec<MessageStatus> {
        self.message_list.into_values().collect()
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

    fn status(kind: &str, code: &str, reason: &str, replacement: bool) -> MessageStatus {
        serde_json::from_value(json!({
            "type": kind, "statusCode": code, "reason": reason, "replacement": replacement
        }))
        .expect("message status")
    }

    fn talk(code: &str, reason: &str) -> MessageStatus {
        status("ATA", code, reason, false)
    }

    fn text(code: &str, reason: &str) -> MessageStatus {
        status("LMS", code, reason, true)
    }

    #[test]
    fn status_classes_follow_code_classes() {
        assert_eq!(StatusClass::of("2000"), StatusClass::InProgress);
        assert_eq!(StatusClass::of("3000"), StatusClass::InProgress);
        assert_eq!(StatusClass::of("4000"), StatusClass::Received);
        for failed in ["1030", "2061", "2230", "3059", "3104", "3107"] {
            assert_eq!(StatusClass::of(failed), StatusClass::Failed, "{failed}");
        }
        // Codes may come as numbers.
        let numeric: MessageStatus =
            serde_json::from_value(json!({ "statusCode": 4000 })).expect("numeric code");
        assert_eq!(numeric.class(), StatusClass::Received);
    }

    #[test]
    fn kakao_results() {
        assert_eq!(
            Delivery::evaluate(&[talk("4000", "수신 완료")]),
            Delivery::Delivered
        );
        assert_eq!(
            Delivery::evaluate(&[talk("2000", "정상 접수")]),
            Delivery::InProgress
        );
        // 3000 = handed to the carrier, report not back yet: not a failure.
        assert_eq!(
            Delivery::evaluate(&[talk("3000", "이통사로 접수")]),
            Delivery::InProgress
        );
        assert_eq!(Delivery::evaluate(&[]), Delivery::InProgress);
        // 2xxx other than 2000 is an error, not "in progress".
        assert_eq!(
            Delivery::evaluate(&[talk("2061", "수신거부")]),
            Delivery::FailedWithoutFallback {
                failed: vec![talk("2061", "수신거부")]
            }
        );
    }

    #[test]
    fn failed_kakao_follows_the_fallback_text() {
        let unreachable = talk("3107", "카카오톡 미사용자");
        // Fallback still with the carrier: wait.
        assert_eq!(
            Delivery::evaluate(&[unreachable.clone(), text("3000", "이통사로 접수")]),
            Delivery::InProgress
        );
        assert_eq!(
            Delivery::evaluate(&[unreachable.clone(), text("4000", "수신 완료")]),
            Delivery::DeliveredByFallback {
                failed: vec![unreachable.clone()],
                fallback: vec![text("4000", "수신 완료")],
            }
        );
        assert_eq!(
            Delivery::evaluate(&[unreachable.clone(), text("3043", "단말기 Power off")]),
            Delivery::Failed {
                failed: vec![unreachable.clone()],
                fallback_failed: vec![text("3043", "단말기 Power off")],
            }
        );
        assert_eq!(unreachable.describe(), "3107 카카오톡 미사용자");
    }

    #[test]
    fn message_list_reads_keyed_messages() {
        let list: MessageListResponse = serde_json::from_value(json!({
            "messageList": {
                "M1": { "type": "ATA", "statusCode": "3105", "reason": "미등록 템플릿", "replacement": false },
                "M2": { "type": "LMS", "statusCode": "4000", "reason": "수신 완료", "replacement": true }
            }
        }))
        .expect("message list");
        let messages = list.into_messages();
        assert_eq!(messages.len(), 2);
        assert!(matches!(
            Delivery::evaluate(&messages),
            Delivery::DeliveredByFallback { .. }
        ));
    }
}
