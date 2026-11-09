pub mod client;
pub mod config;
mod provider;
pub mod types;

use std::time::Duration;

use lettre::{AsyncSmtpTransport, Tokio1Executor, message::Mailbox, transport::smtp::PoolConfig};

use crate::ClientResult;

use config::EmailConfig;

const POOL_SIZE: u32 = 5;
const POOL_IDLE: Duration = Duration::from_secs(60);
const TIMEOUT: Duration = Duration::from_secs(20);

pub struct EmailSender {
    sender: Mailbox,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailSender {
    /// Prepares a pooled STARTTLS connection to the account's SMTP server; nothing is sent yet.
    pub fn new(config: EmailConfig) -> ClientResult<Self> {
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(config.server().host())?
            .credentials(config.credentials())
            .timeout(Some(TIMEOUT))
            .pool_config(
                PoolConfig::new()
                    .max_size(POOL_SIZE)
                    .idle_timeout(POOL_IDLE),
            )
            .build();
        Ok(Self {
            sender: config.sender()?,
            mailer,
        })
    }
}
