use serde::{Deserialize, Deserializer};

use crate::{
    ClientResult,
    errors::ClientError,
    types::{Failure, Outcome},
};

/// Most messages one `send_mass` request takes.
pub const MAX_MASS_MESSAGES: usize = 500;

/// Text message type for `send_mass`. Image messages (MMS) go through `send_image`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextType {
    /// Short text (90 bytes).
    Sms,
    /// Long text (2,000 bytes).
    Lms,
}

impl TextType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sms => "SMS",
            Self::Lms => "LMS",
        }
    }
}

/// One text to one receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextMessage {
    receiver: String,
    body: String,
}

impl TextMessage {
    pub fn new(receiver: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            receiver: receiver.into(),
            body: body.into(),
        }
    }
}

/// `send_mass/` fields after the account fields: `cnt`, `msg_type`, `rec_N` / `msg_N`.
pub(super) fn mass_fields(
    messages: &[TextMessage],
    text_type: TextType,
) -> ClientResult<Vec<(String, String)>> {
    if messages.is_empty() {
        return Err(ClientError::InvalidData("no messages to send"));
    }
    if messages.len() > MAX_MASS_MESSAGES {
        return Err(ClientError::InvalidData(
            "send_mass takes at most 500 messages",
        ));
    }
    let mut fields = vec![
        ("cnt".to_owned(), messages.len().to_string()),
        ("msg_type".to_owned(), text_type.as_str().to_owned()),
    ];
    for (i, message) in messages.iter().enumerate() {
        let n = i + 1;
        fields.push((format!("rec_{n}"), message.receiver.clone()));
        fields.push((format!("msg_{n}"), message.body.clone()));
    }
    Ok(fields)
}

/// `send/` fields for one image message (the image itself is a multipart part).
pub(super) fn image_fields(receiver: &str, message: &str) -> Vec<(String, String)> {
    vec![
        ("receiver".to_owned(), receiver.to_owned()),
        ("msg".to_owned(), message.to_owned()),
        ("msg_type".to_owned(), "MMS".to_owned()),
    ]
}

/// What Aligo accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendReceipt {
    msg_id: Option<String>,
    success_count: i64,
    error_count: i64,
}

impl SendReceipt {
    pub fn msg_id(&self) -> Option<&str> {
        self.msg_id.as_deref()
    }

    pub fn success_count(&self) -> i64 {
        self.success_count
    }

    pub fn error_count(&self) -> i64 {
        self.error_count
    }
}

/// Remaining sendable message counts by type (`remain/`). Aligo reports counts, not money.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Remain {
    sms: i64,
    lms: i64,
    mms: i64,
}

impl Remain {
    pub fn sms(&self) -> i64 {
        self.sms
    }

    pub fn lms(&self) -> i64 {
        self.lms
    }

    pub fn mms(&self) -> i64 {
        self.mms
    }
}

/// Aligo answer. `result_code` is `1` on success and negative otherwise; numbers may come as strings.
#[derive(Debug, Deserialize)]
pub(super) struct AligoResponse {
    #[serde(deserialize_with = "number")]
    result_code: i64,
    #[serde(default)]
    message: String,
    #[serde(default, deserialize_with = "optional_text")]
    msg_id: Option<String>,
    #[serde(default, deserialize_with = "optional_number")]
    success_cnt: Option<i64>,
    #[serde(default, deserialize_with = "optional_number")]
    error_cnt: Option<i64>,
    #[serde(rename = "SMS_CNT", default, deserialize_with = "optional_number")]
    sms_cnt: Option<i64>,
    #[serde(rename = "LMS_CNT", default, deserialize_with = "optional_number")]
    lms_cnt: Option<i64>,
    #[serde(rename = "MMS_CNT", default, deserialize_with = "optional_number")]
    mms_cnt: Option<i64>,
}

impl AligoResponse {
    fn check(&self) -> Result<(), Failure> {
        if self.result_code == 1 {
            Ok(())
        } else {
            Err(Failure::from_message(&self.message))
        }
    }

    pub(super) fn into_send_outcome(self) -> Outcome<SendReceipt> {
        match self.check() {
            Ok(()) => Outcome::Sent(SendReceipt {
                msg_id: self.msg_id,
                success_count: self.success_cnt.unwrap_or(0),
                error_count: self.error_cnt.unwrap_or(0),
            }),
            Err(failure) => Outcome::Failed(failure),
        }
    }

    pub(super) fn into_remain(self) -> Result<Remain, Failure> {
        self.check()?;
        Ok(Remain {
            sms: self.sms_cnt.unwrap_or(0),
            lms: self.lms_cnt.unwrap_or(0),
            mms: self.mms_cnt.unwrap_or(0),
        })
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum NumberOrText {
    Number(i64),
    Text(String),
}

fn number<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
    match NumberOrText::deserialize(deserializer)? {
        NumberOrText::Number(n) => Ok(n),
        NumberOrText::Text(s) => s.trim().parse().map_err(serde::de::Error::custom),
    }
}

fn optional_number<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<i64>, D::Error> {
    Ok(match Option::<NumberOrText>::deserialize(deserializer)? {
        Some(NumberOrText::Number(n)) => Some(n),
        Some(NumberOrText::Text(s)) => s.trim().parse().ok(),
        None => None,
    })
}

fn optional_text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    Ok(match Option::<NumberOrText>::deserialize(deserializer)? {
        Some(NumberOrText::Number(n)) => Some(n.to_string()),
        Some(NumberOrText::Text(s)) => Some(s).filter(|s| !s.is_empty()),
        None => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FailureKind;
    use serde_json::json;

    fn response(value: serde_json::Value) -> AligoResponse {
        serde_json::from_value(value).expect("aligo response")
    }

    #[test]
    fn send_answers_become_outcomes() {
        let sent = response(json!({
            "result_code": "1", "message": "success", "msg_id": 9, "success_cnt": 2, "error_cnt": 0
        }));
        let Outcome::Sent(receipt) = sent.into_send_outcome() else {
            panic!("expected sent")
        };
        assert_eq!(receipt.msg_id(), Some("9"));
        assert_eq!(receipt.success_count(), 2);

        let refused = response(json!({ "result_code": -101, "message": "인증오류입니다." }));
        let Outcome::Failed(failure) = refused.into_send_outcome() else {
            panic!("expected failed")
        };
        assert_eq!(failure.kind(), FailureKind::AuthFailed);
        assert_eq!(failure.detail(), "인증오류입니다.");

        // Any code other than 1 is a refusal, not only -101.
        let other = response(json!({ "result_code": -201, "message": "잔액부족" }));
        assert!(
            matches!(other.into_send_outcome(), Outcome::Failed(f) if f.kind() == FailureKind::LowBalance)
        );
    }

    #[test]
    fn remain_reads_counts() {
        let remain = response(json!({
            "result_code": 1, "message": "", "SMS_CNT": 1200, "LMS_CNT": "390", "MMS_CNT": 168
        }))
        .into_remain()
        .expect("remain");
        assert_eq!((remain.sms(), remain.lms(), remain.mms()), (1200, 390, 168));
    }

    #[test]
    fn mass_fields_number_each_message() {
        let messages = [
            TextMessage::new("01000000000", "a"),
            TextMessage::new("01000000001", "b"),
        ];
        let fields = mass_fields(&messages, TextType::Lms).expect("fields");
        let value = |k: &str| {
            fields
                .iter()
                .find(|(key, _)| key == k)
                .map(|(_, v)| v.as_str())
        };
        assert_eq!(value("cnt"), Some("2"));
        assert_eq!(value("msg_type"), Some("LMS"));
        assert_eq!(value("rec_2"), Some("01000000001"));
        assert_eq!(value("msg_2"), Some("b"));
        assert!(mass_fields(&[], TextType::Sms).is_err());
    }
}
