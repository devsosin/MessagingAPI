# `messaging` — public API

Consumer reference for the `messaging` crate (Kakao 알림톡 and SMS through Solapi/Aligo, email through SMTP). Everything here is `pub` and importable from another crate. Usage examples are in Rust and need an async runtime (tokio). Account values are never shown; only the environment variable **names** are listed.

Maintainers: update this file in the same change as any public API change.

- [Features](#features)
- [Results and errors](#results-and-errors)
- [Solapi](#solapi) · [Aligo](#aligo) · [Email](#email)
- [Versioning](#versioning-and-compatibility)

## Features

Every provider is behind a Cargo feature. Nothing is enabled by default.

```toml
messaging = { git = "https://github.com/devsosin/MessagingAPI", features = ["solapi"] }
# or a local checkout: messaging = { path = "../MessagingAPI", features = ["aligo", "solapi", "email"] }
```

| feature | module | provider | what it does |
|---|---|---|---|
| `solapi` | `messaging::solapi` | Solapi | Kakao 알림톡 send, delivery lookup, balance |
| `aligo` | `messaging::aligo` | Aligo | SMS/LMS, MMS (image), remaining counts |
| `email` | `messaging::email` | SMTP (naver, gmail) | email |

Always available: `messaging::errors` (`ClientError`) and `messaging::ClientResult<T>`. `messaging::types` (`Outcome`, `Failure`, …) exists when `aligo` or `solapi` is enabled. TLS is rustls. All calls are `async`.

## Results and errors

```rust
pub type ClientResult<T> = Result<T, messaging::errors::ClientError>;
```

Three layers; know which one a method returns:

| returned | meaning |
|---|---|
| `Err(ClientError)` | problem **before** anything was sent: missing config, invalid input, client/SMTP setup or transport error |
| `Ok(Outcome::…)` (sends) | what is known about the send — see below |
| `Result<T, Failure>` (lookups) | read-only calls: provider refusal or call failure |

### `ClientError` (`messaging::errors`)

| variant | when |
|---|---|
| `Configuration(&'static str)` | a required env var is missing or empty (carries its name) |
| `InvalidData(&'static str)` | invalid input: empty batch, batch over the limit, unknown `EMAIL_SERVER`, `EMAIL_USERNAME` not an address, invalid email address |
| `ReqwestError(reqwest::Error)` | HTTP client could not be built (`aligo` / `solapi` only) |
| `Smtp(lettre::transport::smtp::Error)` | SMTP refusal or connection problem (`email` only) |
| `EmailMessage(lettre::error::Error)` | email could not be built (`email` only) |

Implements `std::error::Error` and `Debug`. The enum is not `#[non_exhaustive]`; a new variant would be a breaking change.

### `Outcome<T>` (`messaging::types`, `aligo` / `solapi`)

```rust
pub enum Outcome<T> { Sent(T), Failed(Failure), Unknown(UnknownReason) }
```

- `Sent(receipt)`: the provider **accepted** the request. For Solapi this is acceptance only, not delivery.
- `Failed(failure)`: the provider refused it, or the connection never opened, so nothing went out. Inspect `failure.kind()`.
- `Unknown(reason)`: the request may have reached the provider but the result is unknown. **Never retry automatically** — the receiver may get the message twice. Reconcile first (Solapi: look the group up when you have a group id; otherwise check with the provider/your own records), then decide.

`UnknownReason` (implements `Display`):

| variant | meaning |
|---|---|
| `Transport(String)` | failure after the request may have been sent (timeout above all) |
| `UnreadableAnswer(String)` | success response whose body could not be read |
| `NoGroupId` | Solapi answered but gave no group id to look up (Solapi only) |

### `Failure` and `FailureKind` (`messaging::types`)

```rust
impl Failure {
    pub fn kind(&self) -> FailureKind;
    pub fn detail(&self) -> &str;     // provider's own message, for diagnostics/logs
}                                      // also: Display, Debug, Clone, PartialEq, Eq
pub enum FailureKind { LowBalance, AuthFailed, IpNotAllowed, InvalidNumber, Upstream }
impl FailureKind { pub fn is_transient(self) -> bool }   // true only for Upstream
```

`kind` is derived from the provider message text. `Upstream` is the catch-all (provider/network trouble, or an unrecognised message). Retrying a `Failed` send with `is_transient()` is the caller's decision; `Unknown` is never safe to retry blindly.

## Solapi

Feature `solapi`. Module `messaging::solapi`.

**Env vars** for `SolapiConfig::from_env()`: `SOLAPI_API_KEY`, `SOLAPI_API_SECRET`, `SOLAPI_SENDER_NUMBER`, `SOLAPI_PF_ID`. Missing/empty → `ClientError::Configuration(name)`. The sender number must be registered with Solapi; `pf_id` is the Kakao channel id.

### Construction

```rust
// messaging::solapi::config
impl SolapiConfig {
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>,
               sender: impl Into<String>, pf_id: impl Into<String>) -> Self;
    pub fn from_env() -> ClientResult<Self>;
    pub fn to_sender(self) -> ClientResult<Solapi>;      // = Solapi::new(self)
}   // Debug hides key and secret

// messaging::solapi
impl Solapi {
    pub fn new(config: SolapiConfig) -> ClientResult<Self>;                 // default client: 20 s timeout, 5 s connect
    pub fn with_client(config: SolapiConfig, client: reqwest::Client) -> Self;
    pub fn with_base_url(self, base_url: impl Into<String>) -> Self;        // e.g. a mock server
}
```

`with_client` exposes the `reqwest::Client` type: the crate's `reqwest` major (0.13) must match yours.

### Methods on `Solapi`

| method | does |
|---|---|
| `async fn send_alimtalks(&self, messages: &[Alimtalk]) -> ClientResult<Outcome<solapi::types::SendReceipt>>` | Sends alimtalks as one group. `Err(InvalidData)` if empty or more than `MAX_MESSAGES` (10,000). |
| `async fn group_messages(&self, group_id: &str) -> Result<Vec<MessageStatus>, Failure>` | Status of every message in a group, Solapi's fallback texts included. Non-alphanumeric characters of `group_id` are dropped. |
| `async fn balance(&self) -> Result<Balance, Failure>` | Account balance; no side effect, usable as a connection/credential check. |
| `async fn sendable_templates(&self, query: &SendableTemplateQuery) -> Result<Vec<Template>, Failure>` | Templates that can be sent now (`GET kakao/v2/templates/sendable`): approved, not dormant/hidden/deleted. No pagination. Read-only. |
| `async fn template(&self, template_id: &str) -> Result<Template, Failure>` | One template (`GET kakao/v2/templates/:templateId`); Solapi refreshes the dormant state on this call. Characters of `template_id` other than ASCII alphanumerics, `-`, `_` are dropped. Read-only. |

All items below are in `messaging::solapi::types`.

```rust
pub const MAX_MESSAGES: usize = 10_000;

impl Alimtalk {
    pub fn new<K, V>(to: impl Into<String>, template_id: impl Into<String>,
                     variables: impl IntoIterator<Item = (K, V)>) -> Self
    where K: Into<String>, V: Into<String>;
}   // variable names: `회원명` or `#{회원명}` both work

impl SendReceipt {                       // what Solapi accepted
    pub fn group_id(&self) -> &str;                // keep it: needed for group_messages
    pub fn registered(&self) -> i64;               // messages registered in the group
    pub fn refused(&self) -> &[MessageStatus];     // refused at registration
}

impl MessageStatus {                     // one message's status
    pub fn message_id(&self) -> Option<&str>;
    pub fn to(&self) -> Option<&str>;
    pub fn message_type(&self) -> Option<&str>;    // "ATA" (alimtalk), "SMS", "LMS", …
    pub fn status_code(&self) -> &str;
    pub fn status_message(&self) -> &str;
    pub fn is_replacement(&self) -> bool;          // true = Solapi's fallback text for a failed Kakao message
    pub fn describe(&self) -> String;              // "3107 <message>"
    pub fn class(&self) -> StatusClass;
}

pub enum StatusClass { InProgress, Received, Failed }
impl StatusClass { pub fn of(code: &str) -> Self }   // 2000/3000 InProgress, 4000 Received, anything else Failed

pub enum Delivery {
    InProgress,                                                        // something still pending
    Delivered,                                                         // every Kakao message received
    DeliveredByFallback { failed: Vec<MessageStatus>, fallback: Vec<MessageStatus> },
    FailedWithoutFallback { failed: Vec<MessageStatus> },              // may be temporary: fallback can appear a moment later
    Failed { failed: Vec<MessageStatus>, fallback_failed: Vec<MessageStatus> },
}
impl Delivery { pub fn evaluate(messages: &[MessageStatus]) -> Self }

impl Balance {
    pub fn balance(&self) -> f64;
    pub fn point(&self) -> f64;
}

// Kakao templates (read-only lookups; all fields optional on the wire)
#[non_exhaustive]
pub enum TemplateStatus { Pending, Inspecting, Approved, Rejected, Dormant, Other(String) }
// `Other` keeps any value this crate does not know (case-sensitive original text).

impl Template {
    pub fn template_id(&self) -> &str;
    pub fn code(&self) -> Option<&str>;
    pub fn name(&self) -> &str;
    pub fn status(&self) -> &TemplateStatus;
    pub fn is_approved(&self) -> bool;
    pub fn content(&self) -> &str;                 // original text with `#{변수}` placeholders, for previews
    pub fn variable_names(&self) -> Vec<&str>;     // without `#{ }`; usable as Alimtalk::new variable keys
    pub fn buttons(&self) -> &[TemplateButton];
    pub fn channel_id(&self) -> Option<&str>;      // same value as `pf_id`
    pub fn category_code(&self) -> Option<&str>;
    pub fn message_type(&self) -> Option<&str>;
    pub fn is_hidden(&self) -> bool;
    pub fn is_deleted(&self) -> bool;
    pub fn date_created(&self) -> Option<&str>;
    pub fn date_updated(&self) -> Option<&str>;
}

impl TemplateButton {
    pub fn button_type(&self) -> Option<&str>;     // plain string, values not fully documented
    pub fn name(&self) -> Option<&str>;
    pub fn link_mobile(&self) -> Option<&str>;
    pub fn link_pc(&self) -> Option<&str>;
    pub fn link_android(&self) -> Option<&str>;
    pub fn link_ios(&self) -> Option<&str>;
    pub fn chat_extra(&self) -> Option<&str>;
    pub fn opens_outside_kakao(&self) -> Option<bool>;
}

#[derive(Default)]
pub struct SendableTemplateQuery { /* private */ }   // filters; unset/empty ones are not sent, values are URL-encoded
impl SendableTemplateQuery {
    pub fn new() -> Self;
    pub fn channel_id(self, v: impl Into<String>) -> Self;     // = pf_id
    pub fn template_id(self, v: impl Into<String>) -> Self;
    pub fn sender_key(self, v: impl Into<String>) -> Self;
    pub fn template_code(self, v: impl Into<String>) -> Self;
}
```

Template lookups need only the API key and secret. They return `Failure` (no `Outcome`), like the other reads. Sending does not validate variables against the template: compare `variable_names()` with your variables yourself.

```rust
let templates = solapi.sendable_templates(&SendableTemplateQuery::new().channel_id(pf_id)).await?;
for t in &templates { println!("{} {:?} {}", t.name(), t.variable_names(), t.content()); }
```

Decisions use the status **class** only; the fine-grained status codes are not reliable, so use `describe()` for display/logs, not for branching.

### Example

```rust
use messaging::{solapi::{config::SolapiConfig, types::{Alimtalk, Delivery}}, types::Outcome};

let solapi = SolapiConfig::from_env()?.to_sender()?;
let talks = [Alimtalk::new("01012345678", "TEMPLATE_ID", [("회원명", "홍길동")])];

match solapi.send_alimtalks(&talks).await? {
    Outcome::Sent(receipt) => {
        // accepted only; check delivery later (poll until it is not InProgress)
        let messages = solapi.group_messages(receipt.group_id()).await?;
        match Delivery::evaluate(&messages) {
            Delivery::InProgress => { /* ask again later */ }
            Delivery::Delivered | Delivery::DeliveredByFallback { .. } => {}
            Delivery::FailedWithoutFallback { .. } | Delivery::Failed { .. } => { /* report */ }
        }
    }
    Outcome::Failed(failure) => eprintln!("refused: {failure}"),
    Outcome::Unknown(reason) => eprintln!("unknown, do not resend automatically: {reason}"),
}
```

(`?` on a `Failure` needs your own error conversion; `Failure` implements `Display` but not `std::error::Error`.)

Other behaviour: if all messages are refused at registration the outcome is `Failed`; if only some are, it is `Sent` and the refusals are in `receipt.refused()`. Send history is kept 6 months by Solapi.

## Aligo

Feature `aligo`. Module `messaging::aligo`.

**Env vars** for `AligoConfig::from_env()`: `ALIGO_USER_ID`, `ALIGO_API_KEY`, `ALIGO_SENDER_NUMBER`; optional `ALIGO_TEST` (`Y` / `y` = test mode). Aligo also requires the calling IP to be allow-listed (otherwise `FailureKind::IpNotAllowed`).

### Construction

```rust
// messaging::aligo::config
impl AligoConfig {
    pub fn new(user_id: impl Into<String>, api_key: impl Into<String>, sender: impl Into<String>) -> Self;
    pub fn test_mode(self, enabled: bool) -> Self;       // requests are checked, nothing sent or charged
    pub fn from_env() -> ClientResult<Self>;
    pub fn to_sender(self) -> ClientResult<AligoAPI>;    // = AligoAPI::new(self)
}   // Debug hides the API key

// messaging::aligo
impl AligoAPI {
    pub fn new(config: AligoConfig) -> ClientResult<Self>;        // 20 s timeout, 5 s connect
    pub fn with_client(config: AligoConfig, client: reqwest::Client) -> Self;
    pub fn with_base_url(self, base_url: impl Into<String>) -> Self;
}
```

### Methods on `AligoAPI`

| method | does |
|---|---|
| `async fn send_mass(&self, messages: &[TextMessage], text_type: TextType) -> ClientResult<Outcome<aligo::types::SendReceipt>>` | Up to 500 texts in one request. `Err(InvalidData)` if empty or over 500. |
| `async fn send_image(&self, receiver: &str, message: &str, jpeg: Vec<u8>) -> ClientResult<Outcome<SendReceipt>>` | One MMS with a JPEG attached. You supply the bytes; the crate does not download images. |
| `async fn remain(&self) -> Result<Remain, Failure>` | Remaining sendable counts; no side effect. |

Types in `messaging::aligo::types`:

```rust
pub const MAX_MASS_MESSAGES: usize = 500;
pub enum TextType { Sms /* 90 bytes */, Lms /* 2,000 bytes */ }
impl TextMessage { pub fn new(receiver: impl Into<String>, body: impl Into<String>) -> Self }
impl SendReceipt {
    pub fn msg_id(&self) -> Option<&str>;
    pub fn success_count(&self) -> i64;
    pub fn error_count(&self) -> i64;
}
impl Remain { pub fn sms(&self) -> i64; pub fn lms(&self) -> i64; pub fn mms(&self) -> i64 }
```

Aligo's `SendReceipt` is a different type from Solapi's (same name, different module).

### Example

```rust
use messaging::{aligo::{config::AligoConfig, types::{TextMessage, TextType}}, types::Outcome};

let aligo = AligoConfig::from_env()?.to_sender()?;
let texts = [TextMessage::new("01012345678", "안녕하세요")];
match aligo.send_mass(&texts, TextType::Lms).await? {
    Outcome::Sent(r) => println!("ok {} / err {}", r.success_count(), r.error_count()),
    Outcome::Failed(f) => eprintln!("refused: {f}"),
    Outcome::Unknown(r) => eprintln!("unknown, do not resend automatically: {r}"),
}
```

## Email

Feature `email`. Module `messaging::email`. Email returns `ClientResult<()>` — no `Outcome`; failures are `ClientError`.

**Env vars** for `EmailConfig::from_env()`: `EMAIL_SERVER` (`naver` or `gmail`, any case), `EMAIL_USERNAME` (must be an email address; it is also the From address), `EMAIL_SENDER_NAME`, `EMAIL_PASSWORD`. Uses STARTTLS on port 587 with a pool of 5 connections and a 20 s timeout.

```rust
// messaging::email::config
pub enum EmailServer { Naver, Gmail }            // implements FromStr ("naver" | "gmail"), else ClientError::InvalidData
impl EmailConfig {
    pub fn new(server: EmailServer, username: impl Into<String>,
               sender_name: impl Into<String>, password: impl Into<String>) -> Self;
    pub fn from_env() -> ClientResult<Self>;
    pub fn to_sender(self) -> ClientResult<EmailSender>;      // = EmailSender::new(self); nothing is sent yet
}   // Debug hides the password

// messaging::email
impl EmailSender {
    pub fn new(config: EmailConfig) -> ClientResult<Self>;
    pub async fn send_email(&self, email: &Email) -> ClientResult<()>;   // invalid address → Err before connecting
}

// messaging::email::types
pub enum BodyType { Text, Html }
impl Email {
    pub fn new(receiver_email: impl Into<String>, subject: impl Into<String>,
               body: impl Into<String>, body_type: BodyType) -> Self;
    pub fn receiver_name(self, name: impl Into<String>) -> Self;   // display name
}
```

```rust
use messaging::email::{config::{EmailConfig, EmailServer}, types::{BodyType, Email}};

let mailer = EmailConfig::new(EmailServer::Naver, username, "Sender", password).to_sender()?;
let email = Email::new("to@example.com", "제목", "<b>본문</b>", BodyType::Html).receiver_name("홍길동");
mailer.send_email(&email).await?;
```

A timeout or SMTP error does not say whether the mail went out; the same no-blind-retry caution applies.

## Versioning and compatibility

- Semver, pre-1.0 (`0.x`): a breaking change to the public API bumps the **minor** version. Breaking commits are marked `feat!:`; releases are `chore: release X.Y.Z …` commits. Pin an exact minor (`0.3`) and read the release commits before upgrading.
- Changes that break callers include: new variants on the public enums (`Outcome`, `UnknownReason`, `FailureKind`, `ClientError`, `Delivery`, …) because callers match on them exhaustively, signature changes, and removed items.
- Current version: see `version` in `Cargo.toml`. 0.4.0 added template lookups (`sendable_templates`, `template`); 0.3.0 made "outcome unknown" explicit (`Outcome::Unknown(UnknownReason)`); 0.2.0 introduced the current API.
- The README (Korean) is a shorter usage guide; this file is the complete reference.
