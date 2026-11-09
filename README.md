# Messaging API (Email, Kakao alimtalk, SMS)

Rust 클라이언트. 사업자마다 feature 로 나뉘고, 필요한 것만 켠다.

```toml
messaging = { path = "../MessagingAPI", features = ["aligo", "solapi", "email"] }
```

| feature | 사업자 | 용도 |
|---|---|---|
| `aligo` | Aligo | 문자(SMS·LMS), 이미지 문자(MMS), 잔여 건수 |
| `solapi` | Solapi | 카카오 알림톡, 전달 결과 조회, 잔액 |
| `email` | SMTP (naver, gmail) | 이메일 |

TLS 는 rustls 를 쓴다(reqwest, lettre).

## 결과와 오류

- `ClientResult<T>` 의 `Err(ClientError)`: 보내기 전의 문제 — 설정 누락(`Configuration`), 잘못된 입력(`InvalidData`), HTTP 클라이언트·SMTP 오류.
- 문자·알림톡 발송은 `Outcome<SendReceipt>` 를 돌려준다.
  - `Sent(receipt)`: 사업자가 받아들임
  - `Failed(failure)`: 사업자가 거절함 — `failure.kind()`(`LowBalance`, `AuthFailed`, `IpNotAllowed`, `InvalidNumber`, `Upstream`)와 `failure.detail()`(사업자 메시지)
  - `Unknown`: 요청이 닿았을 수 있는데 응답을 못 받음(타임아웃 등). **자동으로 다시 보내지 않는다** — 두 번 발송될 수 있다.
- 조회(`remain`, `balance`, `group_messages`)는 `Result<T, Failure>`.
- 기본 HTTP 클라이언트는 타임아웃 20초(연결 5초). `with_client(config, client)` 로 직접 넘기거나 `with_base_url(url)` 로 다른 주소(목 서버)를 쓸 수 있다.

## Aligo

```rust
use messaging::aligo::{config::AligoConfig, types::{TextMessage, TextType}};

let aligo = AligoConfig::new(user_id, api_key, sender_number).to_sender()?;

let texts = [TextMessage::new("01012345678", "안녕하세요")];
let outcome = aligo.send_mass(&texts, TextType::Lms).await?; // 최대 500건
let outcome = aligo.send_image("01012345678", "쿠폰입니다", jpeg_bytes).await?;
let remain = aligo.remain().await; // 건수: remain.sms(), lms(), mms()
```

- `result_code` 가 1 이 아니면 거절이다.
- 이미지는 JPEG 바이트를 넘긴다(크레이트가 내려받지 않는다).
- `.test_mode(true)`: Aligo 테스트 모드(실제 발송·차감 없음).

## Solapi

```rust
use messaging::solapi::{config::SolapiConfig, types::{Alimtalk, Delivery}};

let solapi = SolapiConfig::new(api_key, api_secret, sender_number, pf_id).to_sender()?;

let talks = [Alimtalk::new("01012345678", template_id, [("회원명", "홍길동")])];
let outcome = solapi.send_alimtalks(&talks).await?; // messages/v4/send-many

// 나중에 전달 결과 확인
let messages = solapi.group_messages(receipt.group_id()).await?;
match Delivery::evaluate(&messages) {
    Delivery::InProgress => {}                         // 아직 진행 중
    Delivery::Delivered => {}                          // 알림톡 수신
    Delivery::DeliveredByFallback { .. } => {}         // 알림톡 실패, 대체 문자 수신
    Delivery::FailedWithoutFallback { .. } => {}       // 알림톡 실패, 대체 문자 없음(아직 없을 수 있음)
    Delivery::Failed { .. } => {}                      // 알림톡·대체 문자 모두 실패
}
```

- 변수 이름은 `회원명` 또는 `#{회원명}` 둘 다 된다.
- `Sent` 는 접수만 뜻한다. 카카오 전달과 Solapi 의 대체 문자 결과는 `group_messages` 로 확인한다.
- 메시지가 전부 접수 단계에서 거절되면 `Failed`, 일부만 거절되면 `Sent` 이고 거절분은 `receipt.refused()`.
- 상태코드는 큰 분류로만 판정한다: `2000`·`3000` 진행 중, `4000` 수신, 그 밖은 실패. 코드와 메시지는 `MessageStatus::describe()` 로 볼 수 있다.
- 발신번호는 사전 등록 필수, 발송 내역 보관 6개월(생성일 기준).

## Email

```rust
use messaging::email::{config::{EmailConfig, EmailServer}, types::{BodyType, Email}};

let mailer = EmailConfig::new(EmailServer::Naver, username, sender_name, password).to_sender()?;
let email = Email::new("to@example.com", "제목", "<b>본문</b>", BodyType::Html).receiver_name("홍길동");
mailer.send_email(&email).await?;
```

STARTTLS(587), 연결 풀 5개.

## Environment Variables

`XxxConfig::from_env()` 가 읽는다. 없으면 `ClientError::Configuration(변수명)`.

```
# Email
EMAIL_SERVER=naver|gmail
EMAIL_USERNAME=
EMAIL_SENDER_NAME=
EMAIL_PASSWORD=

# Solapi
SOLAPI_API_KEY=
SOLAPI_API_SECRET=
SOLAPI_PF_ID=
SOLAPI_SENDER_NUMBER=

# Aligo
ALIGO_API_KEY=
ALIGO_USER_ID=
ALIGO_SENDER_NUMBER=
ALIGO_TEST=Y|N   # 선택
```

## Tests

```sh
cargo test --all-features                                 # 단위 테스트
cargo test --features solapi -- --ignored test_solapi     # 실제 발송 (.env 필요, 비용 발생)
```

실제 사업자를 부르는 테스트(`tests/tester.rs`)는 모두 `#[ignore]` 다.
