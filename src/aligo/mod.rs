pub mod client;
pub mod config;
mod provider;
pub mod types;
mod utils;

use reqwest::Client;

use config::AligoConfig;

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
