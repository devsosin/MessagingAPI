use std::fmt::Debug;

use reqwest::multipart::Form;
use serde::Serialize;

use crate::{
    AligoAPI,
    types::{AligoResponse, ClientResult},
    utils::get_file_part,
};

impl AligoAPI {
    pub(crate) async fn send<T: Serialize + Debug + Into<Form> + Clone>(
        &self,
        uri: &str,
        data: T,
        image_url: Option<&str>,
    ) -> ClientResult<AligoResponse> {
        let body = self.config.to_body(data);

        let builder = self.client.post(format!("https://apis.aligo.in/{uri}"));

        let builder = if let Some(path_or_url) = image_url {
            let part = get_file_part(path_or_url).await?;
            let form: Form = body.into();
            let form = form.part("image", part);
            builder.multipart(form)
        } else {
            builder.form(&body)
        };

        let res = builder.send().await?;

        let response = res.json::<AligoResponse>().await?;
        response.is_error()?;

        Ok(response)
    }
}
