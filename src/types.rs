use std::fmt::Debug;

use reqwest::multipart::Form;
use serde::{Deserialize, Serialize};

use crate::errors::ClientError;

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug, Serialize)]
pub struct AligoRequest<T>
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

    pub(crate) fn get_key(&self) -> &str {
        &self.key
    }
    pub(crate) fn get_userid(&self) -> &str {
        &self.userid
    }
    pub(crate) fn get_sender(&self) -> &str {
        &self.sender
    }
    pub(crate) fn get_testmode(&self) -> String {
        self.testmode_yn.to_string()
    }

    pub(crate) fn get_data(&self) -> &T {
        &self.data
    }
}

impl<T: Serialize + Debug + Into<Form> + Clone> Into<Form> for AligoRequest<T> {
    fn into(self) -> Form {
        let form: Form = self.get_data().clone().into();

        form.text("key", self.get_key().to_owned())
            .text("userid", self.get_userid().to_owned())
            .text("sender", self.get_sender().to_owned())
            .text("testmode_yn", self.get_testmode())
    }
}

#[derive(Debug, Deserialize)]
pub struct AligoResponse {
    success_cnt: Option<i32>,
    result_code: i32,
    message: String,
}

impl AligoResponse {
    pub fn is_error(&self) -> ClientResult<()> {
        match self.result_code {
            -101 => Err(ClientError::AligoError(self.message.clone())),
            _ => Ok(()),
        }
    }
}
