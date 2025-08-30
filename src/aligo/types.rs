use std::fmt::Debug;

use reqwest::multipart::Form;
use serde::{Deserialize, Serialize};

use crate::{ClientResult, errors::ClientError};

#[derive(Debug, Serialize)]
pub(crate) struct AligoRequest<T>
where
    T: Serialize + Debug,
{
    key: String,
    userid: String,
    sender: String,
    testmode_yn: char,
    #[serde(flatten)]
    data: T,
}

impl<T> AligoRequest<T>
where
    T: Serialize + Debug,
{
    pub(crate) fn new(key: &str, userid: &str, sender: &str, testmode_yn: &char, data: T) -> Self {
        Self {
            key: key.into(),
            userid: userid.into(),
            sender: sender.into(),
            testmode_yn: testmode_yn.clone(),
            data,
        }
    }

    pub(crate) fn get_data(&self) -> &T {
        &self.data
    }
}

impl<T: Serialize + Debug + Into<Form> + Clone> Into<Form> for AligoRequest<T> {
    fn into(self) -> Form {
        let form: Form = self.get_data().clone().into();

        form.text("key", self.key.to_owned())
            .text("userid", self.userid.to_owned())
            .text("sender", self.sender.to_owned())
            .text("testmode_yn", self.testmode_yn.to_string())
    }
}

#[derive(Debug, Deserialize)]
pub struct AligoResponse {
    success_cnt: Option<i32>,
    result_code: i32,
    message: String,
}

impl AligoResponse {
    pub(crate) fn is_error(&self) -> ClientResult<()> {
        match self.result_code {
            -101 => Err(ClientError::AligoError(self.message.clone())),
            _ => Ok(()),
        }
    }
}
