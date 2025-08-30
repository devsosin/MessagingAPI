pub mod client;
pub mod config;
mod provider;
pub mod types;

use config::EmailConfig;
use lettre::SmtpTransport;

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
