//! HTTP transport shared by Aligo and Solapi.

use std::{error::Error, time::Duration};

use reqwest::{Client, Response};
use serde::de::DeserializeOwned;

use crate::{
    ClientResult,
    types::{Failure, Outcome, UnknownReason},
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
        Outcome::Unknown(UnknownReason::Transport(describe(err)))
    }
}

/// Reads the body of a send the provider accepted. A body that cannot be read says nothing about
/// what went out.
pub(crate) async fn send_answer<T: DeserializeOwned>(res: Response) -> Result<T, UnknownReason> {
    res.json()
        .await
        .map_err(|err| UnknownReason::UnreadableAnswer(describe(&err)))
}

/// The error with its causes: reqwest keeps the useful part (the timeout, the JSON error) in the
/// source.
fn describe(err: &reqwest::Error) -> String {
    let mut text = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}

/// A read-only call that failed in transport.
pub(crate) fn read_error(err: reqwest::Error) -> Failure {
    Failure::upstream(err.to_string())
}
