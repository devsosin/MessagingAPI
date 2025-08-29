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
        Self {
            config,
            client: Client::new(),
        }
    }
}

pub struct Solapi {
    config: SolapiConfig,
    client: Client,
}

impl Solapi {
    pub fn new(config: SolapiConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

pub struct EmailSender {
    config: EmailConfig,
}
