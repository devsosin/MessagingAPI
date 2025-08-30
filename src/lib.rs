#[cfg(feature = "aligo")]
pub mod aligo;

#[cfg(feature = "email")]
pub mod email;

#[cfg(feature = "solapi")]
pub mod solapi;

// 라이브러리 설계 시 폴더를 feature로 구분하고
// lib에서 해당 feature mod에 #[cfg(feature = "name")]을 설정

// rust-analyzer가 cfg 설정된 코드는 inactive 처리하기 때문에
// vscode에서 User Settings에서 rust-analyzer.cargo.features에 활성화할 feature 추가

pub mod errors;

use crate::errors::ClientError;

pub type ClientResult<T> = Result<T, ClientError>;
