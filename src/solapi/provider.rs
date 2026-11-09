use reqwest::{RequestBuilder, Response, header::AUTHORIZATION};
use serde::de::DeserializeOwned;

use crate::{
    http,
    types::{Failure, Outcome},
};

use super::{
    Solapi,
    types::{ErrorResponse, SendManyBody, SendManyResponse, SendReceipt},
};

impl Solapi {
    fn request(&self, method: reqwest::Method, uri: &str) -> RequestBuilder {
        self.client
            .request(method, format!("{}/{uri}", self.base_url))
            .header(AUTHORIZATION, self.config.authorization())
    }

    /// `messages/v4/send-many`. A refusal comes back as an HTTP error with `errorCode`.
    pub(super) async fn post_send_many(&self, body: &SendManyBody<'_>) -> Outcome<SendReceipt> {
        let res = match self
            .request(reqwest::Method::POST, "messages/v4/send-many")
            .json(body)
            .send()
            .await
        {
            Ok(res) => res,
            Err(err) => return http::send_error(&err),
        };
        if !res.status().is_success() {
            return Outcome::Failed(failure(res).await);
        }
        // Accepted, but an unreadable answer says nothing about what went out.
        match res.json::<SendManyResponse>().await {
            Ok(body) => body.into_outcome(),
            Err(_) => Outcome::Unknown,
        }
    }

    /// A GET without side effects.
    pub(super) async fn get<T: DeserializeOwned>(&self, uri: &str) -> Result<T, Failure> {
        let res = self
            .request(reqwest::Method::GET, uri)
            .send()
            .await
            .map_err(http::read_error)?;
        if !res.status().is_success() {
            return Err(failure(res).await);
        }
        res.json().await.map_err(http::read_error)
    }
}

async fn failure(res: Response) -> Failure {
    let status = res.status().as_u16();
    // An error page that is not JSON still carries the status.
    let body = res.json::<ErrorResponse>().await.unwrap_or_default();
    body.into_failure(status)
}
