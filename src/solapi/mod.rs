pub mod client;
pub mod config;
mod provider;
pub mod types;
mod utils;

use reqwest::Client;

use config::SolapiConfig;

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
