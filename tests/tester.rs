//! Live tests against the real providers. All are `#[ignore]`: they need the accounts in `.env`
//! and may send real messages. Run one with `cargo test --features <name> -- --ignored <test>`.

#[cfg(feature = "solapi")]
mod solapi_tests {
    use messaging::solapi::{config::SolapiConfig, types::Alimtalk};

    /// Sends a real alimtalk with the account in `.env` (charged, the receiver gets it):
    /// `cargo test --features solapi -- --ignored test_solapi`
    #[tokio::test]
    #[ignore = "sends a real message through Solapi"]
    async fn test_solapi() {
        use dotenv::dotenv;
        dotenv().ok();

        let api = SolapiConfig::from_env()
            .expect("solapi env")
            .to_sender()
            .expect("solapi client");

        let talks = [
            Alimtalk::new("00012345678", "test_template", [("회원명", "테스트")]),
            Alimtalk::new("00023456789", "test_template", [("회원명", "테스트")]),
        ];
        let response = api.send_alimtalks(&talks).await;
        println!("{:?}", response);

        let balance = api.balance().await;
        println!("{:?}", balance);
    }

    /// Sends one real alimtalk and follows its group until delivery settles (charged, the receiver
    /// gets it). Besides the `.env` account it needs `SOLAPI_TEST_RECEIVER`, `SOLAPI_TEST_TEMPLATE_ID`
    /// and `SOLAPI_TEST_VARIABLES` (`#{key}=value` pairs separated by `;`):
    /// `cargo test --features solapi -- --ignored test_solapi_delivery --nocapture`
    #[tokio::test]
    #[ignore = "sends a real message through Solapi"]
    async fn test_solapi_delivery() {
        use std::{env, time::Duration};

        use dotenv::dotenv;
        use messaging::{solapi::types::Delivery, types::Outcome};
        dotenv().ok();

        let var = |key: &str| env::var(key).unwrap_or_else(|_| panic!("{key} is not set"));
        let receiver = var("SOLAPI_TEST_RECEIVER");
        let template_id = var("SOLAPI_TEST_TEMPLATE_ID");
        let variables: Vec<(String, String)> = var("SOLAPI_TEST_VARIABLES")
            .split(';')
            .filter_map(|pair| pair.split_once('='))
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();

        let api = SolapiConfig::from_env()
            .expect("solapi env")
            .to_sender()
            .expect("solapi client");
        let talks = [Alimtalk::new(receiver.as_str(), &template_id, variables)];
        let outcome = api.send_alimtalks(&talks).await.expect("send");
        println!("send: {outcome:?}");
        let Outcome::Sent(receipt) = outcome else {
            panic!("not accepted")
        };

        for _ in 0..24 {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let messages = api.group_messages(receipt.group_id()).await.expect("group");
            let delivery = Delivery::evaluate(&messages);
            let codes: Vec<String> = messages.iter().map(|m| m.describe()).collect();
            println!("delivery: {delivery:?} {codes:?}");
            if !matches!(delivery, Delivery::InProgress) {
                return;
            }
        }
        panic!("still in progress after two minutes");
    }
}

#[cfg(feature = "aligo")]
mod aligo_tests {
    use messaging::aligo::{
        config::AligoConfig,
        types::{TextMessage, TextType},
    };

    /// Calls the real Aligo API with the account in `.env` (test mode, nothing is charged):
    /// `cargo test --features aligo -- --ignored test_aligo`
    #[tokio::test]
    #[ignore = "calls the real Aligo API"]
    async fn test_aligo() {
        use dotenv::dotenv;
        dotenv().ok();

        let api = AligoConfig::from_env()
            .expect("aligo env")
            .test_mode(true)
            .to_sender()
            .expect("aligo client");

        let messages = [
            TextMessage::new("00012345678", "Test 1"),
            TextMessage::new("00023456789", "Test 2"),
        ];
        let res = api.send_mass(&messages, TextType::Sms).await;
        println!("{:?}", res);

        let res = api.remain().await;
        println!("{:?}", res);
    }
}

#[cfg(feature = "email")]
mod email_tests {
    use messaging::email::{
        config::EmailConfig,
        types::{BodyType, Email},
    };

    /// Sends a real email with the account in `.env`:
    /// `cargo test --features email -- --ignored test_email`
    #[tokio::test]
    #[ignore = "sends a real email"]
    async fn test_email() {
        use dotenv::dotenv;
        dotenv().ok();

        let mail_sender = EmailConfig::from_env()
            .expect("email env")
            .to_sender()
            .expect("email sender");

        let email = Email::new(
            "test1234@gmail.com",
            "this mail is sent from rust",
            "",
            BodyType::Text,
        )
        .receiver_name("test1");
        let result = mail_sender.send_email(&email).await;
        println!("{:?}", result);
    }
}
