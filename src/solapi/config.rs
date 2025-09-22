use std::env;

use chrono::{SecondsFormat, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::solapi::Solapi;

use super::utils::get_uuid;

pub struct SolapiConfig {
    pf_id: String,
    api_key: String,
    api_secret: String,
    sender_number: String,
}

fn sign(key: &str, msg: &str) -> String {
    let key = key.as_bytes();
    let msg = msg.as_bytes();
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");

    mac.update(msg);
    let result = mac.finalize();
    let code_bytes = result.into_bytes();

    let hmac_string = hex::encode(code_bytes);

    hmac_string
}

impl SolapiConfig {
    pub fn from_env() -> Self {
        let pf_id = env::var("SOLAPI_PF_ID").expect("Failed to get env variable");
        let api_key = env::var("SOLAPI_API_KEY").expect("Failed to get env variable");
        let api_secret = env::var("SOLAPI_API_SECRET").expect("Failed to get env variable");
        let sender_number = env::var("SOLAPI_SENDER_NUMBER").expect("Failed to get env variable");

        Self {
            pf_id,
            api_key,
            api_secret,
            sender_number,
        }
    }

    pub(crate) fn get_authorization(&self) -> String {
        let date_str = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let salt = get_uuid();

        let key = &self.api_secret;
        let msg = &[(&date_str).clone(), (&salt).clone()].join("");
        let signature = sign(key, msg);

        let value = format!(
            "HMAC-SHA256 apiKey={}, date={}, salt={}, signature={}",
            &self.api_key, &date_str, &salt, &signature
        );

        value
    }

    pub(crate) fn get_pf_id(&self) -> &str {
        &self.pf_id
    }
    pub(crate) fn get_from(&self) -> &str {
        &self.sender_number
    }

    pub fn to_sender(self) -> Solapi {
        Solapi::new(self)
    }
}

#[tokio::test]
async fn test_sign() {
    let result = sign("1", "2");
    println!("{}", result)
}

#[tokio::test]
async fn test_utcstr() {
    let dt = Utc::now();

    // ISO 8601 규격 "2025-08-29T07:28:32.858Z"
    println!("{}", dt.to_rfc3339_opts(SecondsFormat::Millis, true));
}
