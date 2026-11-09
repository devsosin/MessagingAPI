use reqwest::{RequestBuilder, multipart};

use crate::{
    ClientResult, http,
    types::{Failure, Outcome},
};

use super::{
    AligoAPI,
    types::{AligoResponse, Remain, SendReceipt},
};

impl AligoAPI {
    fn post(&self, uri: &str) -> RequestBuilder {
        self.client.post(format!("{}/{uri}", self.base_url))
    }

    fn fields(&self, fields: Vec<(String, String)>) -> Vec<(String, String)> {
        let mut all = self.config.auth_fields();
        all.extend(fields);
        all
    }

    /// A form-encoded send (`send_mass/`).
    pub(super) async fn send_form(
        &self,
        uri: &str,
        fields: Vec<(String, String)>,
    ) -> Outcome<SendReceipt> {
        send(self.post(uri).form(&self.fields(fields))).await
    }

    /// A multipart send with one JPEG image (`send/`).
    pub(super) async fn send_with_image(
        &self,
        uri: &str,
        fields: Vec<(String, String)>,
        jpeg: Vec<u8>,
    ) -> ClientResult<Outcome<SendReceipt>> {
        let image = multipart::Part::bytes(jpeg)
            .file_name("image.jpg")
            .mime_str("image/jpeg")?;
        let form = self
            .fields(fields)
            .into_iter()
            .fold(multipart::Form::new(), |form, (k, v)| form.text(k, v))
            .part("image", image);
        Ok(send(self.post(uri).multipart(form)).await)
    }

    /// `remain/` — no side effect.
    pub(super) async fn fetch_remain(&self) -> Result<Remain, Failure> {
        let res = self
            .post("remain/")
            .form(&self.fields(Vec::new()))
            .send()
            .await
            .map_err(http::read_error)?;
        let body: AligoResponse = res.json().await.map_err(http::read_error)?;
        body.into_remain()
    }
}

async fn send(request: RequestBuilder) -> Outcome<SendReceipt> {
    match request.send().await {
        // Aligo answers refusals with HTTP 200 and a negative `result_code`. An answer that cannot be
        // read says nothing about whether the message went out.
        Ok(res) => match res.json::<AligoResponse>().await {
            Ok(body) => body.into_send_outcome(),
            Err(_) => Outcome::Unknown,
        },
        Err(err) => http::send_error(&err),
    }
}
