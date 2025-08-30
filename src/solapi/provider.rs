use std::fmt::Debug;

use reqwest::{StatusCode, header::AUTHORIZATION};
use serde::Serialize;

use crate::ClientResult;

use super::{
    Solapi,
    types::{SolapiErrorResponse, SolapiRequest, SolapiResponse, SolapiSetting},
};

fn is_ok(status_code: &StatusCode) -> bool {
    status_code.as_u16() < 400
}

impl Solapi {
    pub(crate) async fn send_many<T: Serialize + Debug + SolapiSetting>(
        &self,
        uri: &str,
        mut data: T,
    ) -> ClientResult<SolapiResponse<()>> {
        data.set_info(self.config.get_from(), self.config.get_pf_id());
        let body = SolapiRequest::new(data);

        let res = self
            .client
            .post(format!("https://api.solapi.com/{uri}"))
            .header(AUTHORIZATION, self.config.get_authorization())
            .json(&body)
            .send()
            .await?;

        let result: ClientResult<SolapiResponse<()>> = match is_ok(&res.status()) {
            true => res.json::<SolapiResponse<()>>().await.map_err(|e| e.into()),
            false => {
                let err_response = res.json::<SolapiErrorResponse>().await.unwrap();

                Err(err_response.into())
            }
        };

        // println!("{:?}", result);

        Ok(result?)
    }
}
