use serde::Serialize;
use std::{collections::HashMap, iter::zip};

use crate::{ClientResult, errors::ClientError};

use super::{
    Solapi,
    types::{SolapiResponse, SolapiSetting, ToAlimtalkVariable},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct KakaoBody {
    allow_duplicates: bool,
    messages: Vec<SolapiMessage>,
}

impl SolapiSetting for KakaoBody {
    fn set_info(&mut self, from: &str, pf_id: &str) {
        self.messages.iter_mut().for_each(|m| {
            m.set_from(from);
            m.set_pf_id(pf_id);
        });
    }
}

#[tokio::test]
async fn test_iter_mut() {
    let mut s = vec!["1".to_string(), "2".to_string()];

    s.iter_mut().for_each(|c| c.push_str("abc"));

    println!("{:?}", s);
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SolapiMessage {
    to: String,
    from: Option<String>,
    // 친구톡, 알림톡
    kakao_options: KakaoOption,
}

impl SolapiMessage {
    fn set_from(&mut self, from: &str) {
        self.from = Some(from.into());
    }
    fn set_pf_id(&mut self, pf_id: &str) {
        self.kakao_options.pf_id = Some(pf_id.into())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct KakaoOption {
    pf_id: Option<String>,
    template_id: String,
    variables: HashMap<String, String>, // "#{키}"": "값"
}

pub trait SolapiMessaging {
    async fn send_alimtalks<T: ToAlimtalkVariable>(
        &self,
        template_id: &str,
        receivers: &Vec<String>,
        variables: &Vec<T>,
    ) -> ClientResult<SolapiResponse<()>>;
}

impl SolapiMessaging for Solapi {
    async fn send_alimtalks<T: ToAlimtalkVariable>(
        &self,
        template_id: &str,
        receivers: &Vec<String>,
        variables: &Vec<T>,
    ) -> ClientResult<SolapiResponse<()>> {
        let uri = "messages/v4/send-many/detail";
        let mut messages = vec![];

        for (receiver, variable) in zip(receivers, variables) {
            if !(receiver.starts_with("10") | receiver.starts_with("010")) {
                continue;
            }

            let option = KakaoOption {
                pf_id: None,
                template_id: template_id.into(),
                variables: variable.to_map(),
            };

            let message = SolapiMessage {
                to: receiver.replace("-", ""),
                from: None,
                kakao_options: option,
            };

            messages.push(message);
        }

        if messages.len() == 0 {
            return Err(ClientError::SolapiError("No Messages To Send".into()));
        }

        let body = KakaoBody {
            allow_duplicates: true,
            messages,
        };

        self.send_many(uri, body).await
    }
}
