pub mod clients;
pub mod configs;
pub mod errors;
pub mod provider;
pub mod types;
pub mod utils;

use reqwest::Client;

use crate::configs::{aligo::AligoConfig, email::EmailConfig, solapi::SolapiConfig};

pub struct AligoAPI {
    config: AligoConfig,
    client: Client,
}

impl AligoAPI {
    pub fn new(config: AligoConfig) -> Self {
        let client = Client::new();

        Self { config, client }
    }
}

pub struct Solapi {
    config: SolapiConfig,
    client: Client,
}

pub struct EmailSender {
    config: EmailConfig,
}
