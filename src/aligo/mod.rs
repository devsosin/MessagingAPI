pub mod client;
pub mod config;
mod provider;
pub mod types;

use reqwest::Client;

use crate::{ClientResult, http};

use config::AligoConfig;

const BASE_URL: &str = "https://apis.aligo.in";

pub struct AligoAPI {
    config: AligoConfig,
    client: Client,
    base_url: String,
}

impl AligoAPI {
    /// Uses the crate's default HTTP client (with timeouts).
    pub fn new(config: AligoConfig) -> ClientResult<Self> {
        Ok(Self::with_client(config, http::default_client()?))
    }

    pub fn with_client(config: AligoConfig, client: Client) -> Self {
        Self {
            config,
            client,
            base_url: BASE_URL.into(),
        }
    }

    /// Points the API at another host, e.g. a mock server in tests.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').into();
        self
    }
}
