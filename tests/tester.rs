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
    use messaging::solapi::{
        Solapi, client::SolapiMessaging, config::SolapiConfig, types::ToAlimtalkVariable,
    };
    use rustc_version::version;
    use std::{collections::HashMap, env::consts};

    #[tokio::test]
    async fn test_os() {
        let os_info = consts::OS;
        let version = version().unwrap();

        println!("{}", os_info);
        println!("{}", version);
    }

    #[tokio::test]
    async fn test_solapi() {
        use dotenv::dotenv;
        dotenv().ok();

        let config = SolapiConfig::from_env();
        let api = Solapi::new(config);
        let receivers = vec!["00012345678".to_string(), "00023456789".to_string()];
        let template_id = "test_template";
        let mut variable1 = HashMap::new();
        variable1.insert("#{회원명}".into(), "테스트".into());
        let mut variable2 = HashMap::new();
        variable2.insert("#{회원명}".into(), "테스트".into());

        let variables = vec![variable1, variable2];

        let response = api
            .send_alimtalks(template_id, &receivers, &variables)
            .await;

        println!("{:?}", response);
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
