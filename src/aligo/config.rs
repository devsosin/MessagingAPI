use std::{env, fmt::Debug};

use serde::Serialize;

use crate::aligo::AligoAPI;

use super::types::AligoRequest;

#[derive(Debug)]
pub struct AligoConfig {
    user_id: String,
    api_key: String,
    sender_number: String,
    is_test: char, // Y/N
}

impl AligoConfig {
    pub fn from_env() -> Self {
        let user_id = env::var("ALIGO_USER_ID").expect("Failed to get env variable");
        let api_key = env::var("ALIGO_API_KEY").expect("Failed to get env variable");
        let sender_number = env::var("ALIGO_SENDER_NUMBER").expect("Failed to get env variable");
        let is_test = env::var("ALIGO_TEST")
            .expect("Failed to get env variable")
            .parse()
            .unwrap();

        Self {
            user_id,
            api_key,
            sender_number,
            is_test,
        }
    }

    pub(super) fn to_body<T: Serialize + Debug>(&self, data: T) -> AligoRequest<T> {
        AligoRequest::new(
            &self.api_key,
            &self.user_id,
            &self.sender_number,
            &self.is_test,
            data,
        )
    }

    pub fn to_sender(self) -> AligoAPI {
        AligoAPI::new(self)
    }
}
