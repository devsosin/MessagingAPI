//! Results shared by the HTTP providers (Aligo, Solapi).

use std::fmt;

/// Why a provider refused a request or could not be reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    LowBalance,
    AuthFailed,
    IpNotAllowed,
    InvalidNumber,
    /// The provider or the network had a problem; nothing points at the request itself.
    Upstream,
}

impl FailureKind {
    /// Classifies a provider error message (Korean or English).
    pub(crate) fn from_message(message: &str) -> Self {
        let m = message.to_lowercase();
        if m.contains("ip") {
            Self::IpNotAllowed
        } else if m.contains("인증")
            || m.contains("key")
            || m.contains("signature")
            || m.contains("unauthorized")
        {
            Self::AuthFailed
        } else if m.contains("잔액")
            || m.contains("포인트")
            || m.contains("balance")
            || m.contains("충전")
        {
            Self::LowBalance
        } else if m.contains("수신")
            || m.contains("번호")
            || m.contains("receiver")
            || m.contains("invalid to")
        {
            Self::InvalidNumber
        } else {
            Self::Upstream
        }
    }

    /// Worth retrying automatically: the provider or the network had a moment.
    pub fn is_transient(self) -> bool {
        self == Self::Upstream
    }
}

/// A refusal or a failed call, with the provider's own message for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    kind: FailureKind,
    detail: String,
}

impl Failure {
    pub(crate) fn new(kind: FailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    /// Classifies the provider message with [`FailureKind::from_message`] and keeps it as the detail.
    pub(crate) fn from_message(message: &str) -> Self {
        Self::new(FailureKind::from_message(message), message)
    }

    pub(crate) fn upstream(detail: impl Into<String>) -> Self {
        Self::new(FailureKind::Upstream, detail)
    }

    pub fn kind(&self) -> FailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.detail)
    }
}

/// Outcome of a send. `Unknown` = the request may have reached the provider but no answer came back
/// (a timeout); never retry it automatically, or the receiver may get the message twice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome<T> {
    Sent(T),
    Failed(Failure),
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_messages_map_to_failure_kinds() {
        assert_eq!(
            FailureKind::from_message("등록되지 않은 IP 입니다"),
            FailureKind::IpNotAllowed
        );
        assert_eq!(
            FailureKind::from_message("인증오류입니다.-API Key"),
            FailureKind::AuthFailed
        );
        assert_eq!(
            FailureKind::from_message("잔액이 부족합니다"),
            FailureKind::LowBalance
        );
        assert_eq!(
            FailureKind::from_message("수신번호 오류"),
            FailureKind::InvalidNumber
        );
        assert_eq!(
            FailureKind::from_message("알 수 없는 오류"),
            FailureKind::Upstream
        );
        assert!(FailureKind::Upstream.is_transient());
        assert!(!FailureKind::AuthFailed.is_transient());
    }
}
