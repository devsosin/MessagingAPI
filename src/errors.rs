use thiserror::Error;

/// Errors raised before or around a provider call: configuration, invalid input and transport setup.
/// A provider refusing a request is not an error here; it is a [`crate::types::Failure`].
#[derive(Error, Debug)]
pub enum ClientError {
    #[cfg(any(feature = "aligo", feature = "solapi"))]
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Configuration missing: {0}")]
    Configuration(&'static str),
    #[error("Invalid data: {0}")]
    InvalidData(&'static str),

    #[cfg(feature = "email")]
    #[error("SMTP error: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
    #[cfg(feature = "email")]
    #[error("Email message error: {0}")]
    EmailMessage(#[from] lettre::error::Error),
}
