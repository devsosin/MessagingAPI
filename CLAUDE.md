# MessagingAPI — `messaging` crate

Rust library (crate `messaging`, edition 2024) that sends messages through Korean providers. Each provider sits behind a Cargo feature, so consumers enable only what they need. GitHub: `devsosin/MessagingAPI`. User-facing docs are `README.md` (Korean) — it is the usage reference; do not duplicate it here.

## Layout

```text
src/
  lib.rs        feature-gated `pub mod`s; `ClientResult<T> = Result<T, ClientError>`
  errors.rs     ClientError (pre-send problems: Configuration, InvalidData, HTTP/SMTP)
  types.rs      shared Outcome<SendReceipt> / Failure / UnknownReason (aligo + solapi)
  http.rs       default reqwest client (20s timeout, 5s connect)   [private]
  env.rs        required()/optional() env readers                 [private]
  aligo/        feature `aligo`  — SMS/LMS/MMS, remaining count
  solapi/       feature `solapi` — Kakao 알림톡 send, delivery lookup, balance
  email/        feature `email`  — SMTP via lettre (naver, gmail)
tests/
  outcome.rs    unit-style tests (mock server, no network to providers)
  tester.rs     live tests, all `#[ignore]`
docs/RUST_CODE_STYLE.md   code style guide
.claude/github-issue.yml  issue settings (see harness:github-issue)
```

Each provider module has the same shape: `config.rs` (`XxxConfig::new` / `from_env` / `to_sender`), `client.rs` (public methods), `provider.rs` (private request/response plumbing), `types.rs` (requests, responses, enums), `mod.rs` (the client struct, `new`, `with_client`, `with_base_url`).

## Public API surface

- `messaging::aligo::{config::AligoConfig, client…}`: `send_mass`, `send_image`, `remain`.
- `messaging::solapi::{config::SolapiConfig, types::{Alimtalk, Delivery, MessageStatus}}`: `send_alimtalks`, `group_messages`, `balance`, `Delivery::evaluate`.
- `messaging::email::{config::EmailConfig, types::{Email, BodyType}}`: `send_email`.
- `messaging::types::Outcome` (`Sent` / `Failed` / `Unknown`) for sends; lookups return `Result<T, Failure>`.
- Never auto-retry an `Unknown` outcome — the request may have gone through (double send). This is a deliberate contract; keep it.

## Build and test

```sh
cargo build --all-features
cargo test --all-features                                  # unit tests; live tests are skipped (#[ignore])
cargo test --features solapi -- --ignored test_solapi      # LIVE: real send, costs money
```

- `cargo test --all-features` is the `verify` command (also in `.claude/github-issue.yml`). Verified passing in the task container (the 4 live tests show as ignored).
- Live tests in `tests/tester.rs` load `.env` (git-ignored; never print or commit values). Key names:
  - Solapi: `SOLAPI_API_KEY`, `SOLAPI_API_SECRET`, `SOLAPI_PF_ID`, `SOLAPI_SENDER_NUMBER`; the delivery test also needs `SOLAPI_TEST_RECEIVER`, `SOLAPI_TEST_TEMPLATE_ID`, `SOLAPI_TEST_VARIABLES`.
  - Aligo: `ALIGO_API_KEY`, `ALIGO_USER_ID`, `ALIGO_SENDER_NUMBER`, optional `ALIGO_TEST`.
  - Email: `EMAIL_SERVER`, `EMAIL_USERNAME`, `EMAIL_SENDER_NAME`, `EMAIL_PASSWORD`.
- Confirm with the user before running any `--ignored` test: they hit real provider APIs, send real messages and are billed.
- rust-analyzer shows feature-gated code as inactive unless `rust-analyzer.cargo.features` lists the features.

## Code style

Follow `docs/RUST_CODE_STYLE.md`: construct via `::new()`, private fields, enums in `types.rs` for fixed value sets, typed errors, no `unwrap` on external data, explicit unknown-outcome handling. Provider-specific helpers stay inside the provider module.

## Versioning and git

- Issue ids are `MSG-NNN`; issue title format `[MSG-NNN] summary`. Labels: `provider:aligo|solapi|email` plus type labels.
- Commits: Conventional Commits with the issue id in parentheses at the end, e.g. `feat(solapi): send through messages/v4/send-many/detail (MSG-002)`. Breaking API changes use `feat!:`.
- Releases are a separate `chore: release X.Y.Z …` commit that bumps `version` in `Cargo.toml` (currently 0.3.0; pre-1.0, breaking changes bump the minor) and updates README. Work lands via PR branches (e.g. `fix/msg-002-solapi-group-id`) merged with merge commits.
- `Cargo.lock` is git-ignored (library).

## Planned: 알림톡 template lookup (not implemented)

Likely home: `src/solapi/` — a method on `Solapi` in `client.rs`, request plumbing in `provider.rs`, template types/enums in `types.rs`, returning `Result<T, Failure>` like `group_messages`/`balance`. Document it in the README's Solapi section and add a live test (`#[ignore]`) in `tests/tester.rs`. Check the Solapi Kakao template API docs before designing.
