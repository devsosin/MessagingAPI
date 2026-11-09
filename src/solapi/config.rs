use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use time::{OffsetDateTime, format_description::well_known::Rfc3339, macros::offset};
use uuid::Uuid;

use crate::{ClientResult, env, solapi::Solapi};

/// Solapi account values. The key and secret are never logged or shown by `Debug`.
pub struct SolapiConfig {
    api_key: String,
    api_secret: String,
    sender: String,
    pf_id: String,
}

impl SolapiConfig {
    /// `sender` = registered sender number, `pf_id` = Kakao channel (plus friend) id.
    pub fn new(
        api_key: impl Into<String>,
        api_secret: impl Into<String>,
        sender: impl Into<String>,
        pf_id: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
            sender: sender.into(),
            pf_id: pf_id.into(),
        }
    }

    /// `SOLAPI_API_KEY`, `SOLAPI_API_SECRET`, `SOLAPI_SENDER_NUMBER`, `SOLAPI_PF_ID`.
    pub fn from_env() -> ClientResult<Self> {
        Ok(Self::new(
            env::required("SOLAPI_API_KEY")?,
            env::required("SOLAPI_API_SECRET")?,
            env::required("SOLAPI_SENDER_NUMBER")?,
            env::required("SOLAPI_PF_ID")?,
        ))
    }

    pub fn to_sender(self) -> ClientResult<Solapi> {
        Solapi::new(self)
    }

    pub(super) fn sender(&self) -> &str {
        &self.sender
    }

    pub(super) fn pf_id(&self) -> &str {
        &self.pf_id
    }

    /// `HMAC-SHA256 ApiKey=…, Date=…, salt=…, signature=hex(HMAC(secret, date + salt))`, with a
    /// fresh date and salt on every request.
    pub(super) fn authorization(&self) -> String {
        let date = OffsetDateTime::now_utc()
            .to_offset(offset!(+9))
            .format(&Rfc3339)
            .unwrap_or_default();
        let salt = Uuid::new_v4().simple().to_string();
        let signature = sign(&self.api_secret, &format!("{date}{salt}"));
        format!(
            "HMAC-SHA256 ApiKey={}, Date={date}, salt={salt}, signature={signature}",
            self.api_key
        )
    }
}

impl std::fmt::Debug for SolapiConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolapiConfig")
            .field("sender", &self.sender)
            .field("pf_id", &self.pf_id)
            .finish_non_exhaustive()
    }
}

fn sign(secret: &str, message: &str) -> String {
    // HMAC accepts keys of any length, so this never fails.
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return String::new();
    };
    mac.update(message.as_bytes());
    mac.finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_is_hex_hmac_sha256() {
        // RFC 4231 test case 2.
        assert_eq!(
            sign("Jefe", "what do ya want for nothing?"),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn authorization_carries_key_date_salt_and_signature() {
        let config = SolapiConfig::new("KEY", "SECRET", "0200000000", "PF");
        let header = config.authorization();
        assert!(header.starts_with("HMAC-SHA256 ApiKey=KEY, Date="));
        assert!(header.contains("+09:00, salt="));
        assert!(!header.contains("SECRET"));
        assert!(!format!("{config:?}").contains("KEY"));
    }
}
