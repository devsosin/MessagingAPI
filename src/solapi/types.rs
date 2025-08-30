use std::{collections::HashMap, env::consts::OS, fmt::Debug};

use rustc_version::version;
use serde::{Deserialize, Serialize};

use crate::errors::ClientError;

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
