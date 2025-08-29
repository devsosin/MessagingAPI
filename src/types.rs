use std::{collections::HashMap, env::consts::OS, fmt::Debug, io::Error};

use reqwest::multipart::Form;
use rustc_version::version;
use serde::{Deserialize, Serialize};

use crate::errors::ClientError;

pub type ClientResult<T> = Result<T, ClientError>;

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

#[derive(Debug, Serialize)]
pub(crate) struct SolapiRequest<T> {
    agent: AgentInfo,

    #[serde(flatten)]
    data: T,
}

impl<T: Serialize + Debug> SolapiRequest<T> {
    pub(crate) fn new(data: T) -> Self {
        Self {
            agent: AgentInfo::new(),
            data,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentInfo {
    sdk_version: String,
    os_platform: String,
}

impl AgentInfo {
    fn new() -> Self {
        let pkg_version = env!("CARGO_PKG_VERSION");

        Self {
            sdk_version: format!("rust/{pkg_version}"),
            os_platform: format!("{} | {}", OS, version().unwrap()),
        }
    }
}

pub(crate) trait SolapiSetting {
    fn set_info(&mut self, from: &str, pf_id: &str);
}

pub trait ToAlimtalkVariable {
    fn to_map(&self) -> HashMap<String, String>;
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolapiResponse<T> {
    status_code: i32,
    status_message: String,
    data: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SolapiErrorResponse {
    error_code: String,
    error_message: String,
}

impl Into<ClientError> for SolapiErrorResponse {
    fn into(self) -> ClientError {
        // println!("{}", self.error_code);
        ClientError::SolapiError(self.error_message)
    }
}

pub trait EmailTemplateLoader {
    async fn get_content(&self) -> Result<String, Error>;
    fn is_html(&self) -> bool;
}

pub trait ReceiverGetter {
    fn get_name(&self) -> &Option<String>;
    fn get_address(&self) -> &str;
}

pub trait ToEmailVariable {
    fn to_map(&self) -> HashMap<String, String>;
}
