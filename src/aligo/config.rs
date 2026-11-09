use crate::{ClientResult, aligo::AligoAPI, env};

/// Aligo account values. The API key is never logged or shown by `Debug`.
pub struct AligoConfig {
    user_id: String,
    api_key: String,
    sender: String,
    test_mode: bool,
}

impl AligoConfig {
    pub fn new(
        user_id: impl Into<String>,
        api_key: impl Into<String>,
        sender: impl Into<String>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            api_key: api_key.into(),
            sender: sender.into(),
            test_mode: false,
        }
    }

    /// Aligo test mode: requests are checked and answered but nothing is sent or charged.
    pub fn test_mode(mut self, enabled: bool) -> Self {
        self.test_mode = enabled;
        self
    }

    /// `ALIGO_USER_ID`, `ALIGO_API_KEY`, `ALIGO_SENDER_NUMBER`, optional `ALIGO_TEST` (`Y` = test mode).
    pub fn from_env() -> ClientResult<Self> {
        let config = Self::new(
            env::required("ALIGO_USER_ID")?,
            env::required("ALIGO_API_KEY")?,
            env::required("ALIGO_SENDER_NUMBER")?,
        );
        let test_mode = env::optional("ALIGO_TEST").is_some_and(|v| v.eq_ignore_ascii_case("y"));
        Ok(config.test_mode(test_mode))
    }

    pub fn to_sender(self) -> ClientResult<AligoAPI> {
        AligoAPI::new(self)
    }

    /// Account fields every Aligo request starts with.
    pub(super) fn auth_fields(&self) -> Vec<(String, String)> {
        let mut fields = vec![
            ("key".to_owned(), self.api_key.clone()),
            ("userid".to_owned(), self.user_id.clone()),
            ("sender".to_owned(), self.sender.clone()),
        ];
        if self.test_mode {
            fields.push(("testmode_yn".to_owned(), "Y".to_owned()));
        }
        fields
    }
}

impl std::fmt::Debug for AligoConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AligoConfig")
            .field("user_id", &self.user_id)
            .field("sender", &self.sender)
            .field("test_mode", &self.test_mode)
            .finish_non_exhaustive()
    }
}
