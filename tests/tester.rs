use serde_json::json;

#[tokio::test]
async fn test_serde() {
    let value = json!({
        "test": 'Y'
    });

    println!("{value}");
}

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
    use messaging::email::{EmailSender, client::EmailMessaging, config::EmailConfig};
    use std::collections::HashMap;
    use tokio::fs;

    #[tokio::test]
    async fn test_email() {
        use dotenv::dotenv;
        dotenv().ok();

        let mail_sender = EmailConfig::from_env().to_sender();

        let subject = "this mail is sent from rust";
        let content = "";

        // Send the email
        let result = mail_sender
            .send_email(
                &Some("test1".into()),
                "test1234@gmail.com",
                subject,
                content,
                false,
            )
            .await;
        println!("{:?}", result);
    }
}
