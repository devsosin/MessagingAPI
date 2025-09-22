pub mod client;
pub mod config;
mod provider;

use std::time::Duration;

use config::EmailConfig;
use lettre::{AsyncSmtpTransport, Tokio1Executor, transport::smtp::PoolConfig};

pub struct EmailSender {
    config: EmailConfig,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailSender {
    pub fn new(config: EmailConfig) -> Self {
        let creds = (&config).into();
        let mail_server: &str = config.get_server().into();
        let smtp_server = format!("smtp.{}.com", mail_server);

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_server)
            .unwrap()
            .credentials(creds)
            .pool_config(
                PoolConfig::new()
                    .max_size(5) // Maximum Pool
                    .idle_timeout(Duration::from_secs(60)), // idle time
            )
            .build();

        Self { config, mailer }
    }
}
