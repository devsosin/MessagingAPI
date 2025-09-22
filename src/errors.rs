use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Aligo Error: {0}")]
    AligoError(String),
    #[error("Solapi Error: {0}")]
    SolapiError(String),
    #[error("Email Error: {0}")]
    EmailError(String),

    #[error("Internal error: {0}")]
    InternalError(&'static str),
}
