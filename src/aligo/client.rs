use std::{collections::HashMap, iter::zip};

use reqwest::multipart;
use serde::Serialize;

use crate::ClientResult;

use super::{AligoAPI, types::AligoResponse};

enum MessageType {
    SMS,
    LMS,
    MMS,
}

impl Into<String> for MessageType {
    fn into(self) -> String {
        let v = match self {
            MessageType::SMS => "SMS",
            MessageType::LMS => "LMS",
            MessageType::MMS => "MMS",
        };

        v.to_uppercase()
    }
}

impl From<&str> for MessageType {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "sms" => Self::SMS,
            "lms" => Self::LMS,
            "mms" => Self::MMS,
            _ => panic!("Unknown Message Type"),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
struct SmsBody {
    cnt: usize,
    msg_type: String, // SMS, LMS
    #[serde(flatten)]
    msg_body: HashMap<String, String>,
}

impl Into<multipart::Form> for SmsBody {
    fn into(self) -> multipart::Form {
        multipart::Form::new()
    }
}

#[derive(Debug, Serialize, Clone)]
struct MmsBody {
    receiver: String,
    msg: String,
    msg_type: String,
}

impl Into<multipart::Form> for MmsBody {
    fn into(self) -> multipart::Form {
        multipart::Form::new()
            .text("receiver", self.receiver.to_owned())
            .text("msg", self.msg.to_owned())
            .text("msg_type", self.msg_type.to_owned())
    }
}

pub trait AligoMessaging {
    async fn send_sms(
        &self,
        receivers: &Vec<&str>,
        messages: &Vec<&str>,
        message_type: &str,
    ) -> ClientResult<AligoResponse>;
    async fn send_mms(
        &self,
        receiver: &str,
        message: &str,
        image_path: &str,
    ) -> ClientResult<AligoResponse>;
}

impl AligoMessaging for AligoAPI {
    async fn send_sms(
        &self,
        receivers: &Vec<&str>,
        messages: &Vec<&str>,
        message_type: &str,
    ) -> ClientResult<AligoResponse> {
        let msg_type: MessageType = message_type.into();

        let mut msg_body = HashMap::new();

        for (i, (rec, msg)) in zip(receivers, messages).enumerate() {
            let i = i + 1;
            msg_body.insert(format!("rec_{i}"), rec.to_string());
            msg_body.insert(format!("msg_{i}"), msg.to_string());
        }

        let data = SmsBody {
            cnt: msg_body.len() / 2,
            msg_type: msg_type.into(),
            msg_body,
        };

        self.send("send_mass/", data, None).await
    }

    async fn send_mms(
        &self,
        receiver: &str,
        message: &str,
        image_path: &str,
    ) -> ClientResult<AligoResponse> {
        let msg_type = MessageType::MMS;

        let data = MmsBody {
            receiver: receiver.into(),
            msg: message.into(),
            msg_type: msg_type.into(),
        };

        self.send("send/", data, Some(image_path)).await
    }
}
