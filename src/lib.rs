pub mod clients;
pub mod configs;
pub mod errors;
pub mod provider;
pub mod types;
pub mod utils;

use lettre::SmtpTransport;
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
    mailer: SmtpTransport,
}

impl EmailSender {
    pub fn new(config: EmailConfig) -> Self {
        let creds = (&config).into();
        let mail_server: &str = config.get_server().into();
        let smtp_server = format!("smtp.{}.com", mail_server);

        // pool: Maximum 10, idle time: 60s
        let mailer = SmtpTransport::relay(&smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

        Self { config, mailer }
    }
}
