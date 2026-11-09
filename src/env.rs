use std::env;

use crate::{ClientResult, errors::ClientError};

/// A required environment variable; missing or empty → [`ClientError::Configuration`].
pub(crate) fn required(name: &'static str) -> ClientResult<String> {
    optional(name).ok_or(ClientError::Configuration(name))
}

/// An optional environment variable; empty counts as missing.
pub(crate) fn optional(name: &str) -> Option<String> {
    env::var(name).ok().filter(|v| !v.trim().is_empty())
}
