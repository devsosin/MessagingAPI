//! HTTP transport shared by Aligo and Solapi.

use std::time::Duration;

use reqwest::Client;

use crate::{
    ClientResult,
    types::{Failure, Outcome},
};

const TIMEOUT: Duration = Duration::from_secs(20);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// The client used when the caller does not pass its own: every call is bounded by a timeout.
pub(crate) fn default_client() -> ClientResult<Client> {
    Ok(Client::builder()
        .timeout(TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .build()?)
}

/// A send whose request failed in transport. A connection that never opened sent nothing; any other
/// error (a timeout above all) may have reached the provider, so the outcome is unknown.
pub(crate) fn send_error<T>(err: &reqwest::Error) -> Outcome<T> {
    if err.is_connect() {
        Outcome::Failed(Failure::upstream(err.to_string()))
    } else {
        Outcome::Unknown
    }
}

/// A read-only call that failed in transport.
pub(crate) fn read_error(err: reqwest::Error) -> Failure {
    Failure::upstream(err.to_string())
}
