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

    #[error("Aligo Error: {0}")]
    AligoError(String),
    #[error("Solapi Error: {0}")]
    SolapiError(String),
    #[error("Email Error: {0}")]
    EmailError(String),

    #[error("Internal error: {0}")]
    InternalError(&'static str),
}
