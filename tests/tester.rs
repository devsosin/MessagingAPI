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

#[cfg(feature = "solapi")]
mod solapi_template_tests {
    use messaging::solapi::{config::SolapiConfig, types::SendableTemplateQuery};

    fn api() -> messaging::solapi::Solapi {
        dotenv::dotenv().ok();
        // Lookups need only the key and secret (not the sender number or `pf_id`).
        let var = |key: &str| std::env::var(key).unwrap_or_else(|_| panic!("{key} is not set"));
        SolapiConfig::new(var("SOLAPI_API_KEY"), var("SOLAPI_API_SECRET"), "", "")
            .to_sender()
            .expect("solapi client")
    }

    /// Read-only: lists the sendable templates and prints names' status and variables only.
    /// `cargo test --features solapi -- --ignored test_template_sendable --nocapture`
    #[tokio::test]
    #[ignore = "calls the real Solapi API"]
    async fn test_template_sendable() {
        let templates = api()
            .sendable_templates(&SendableTemplateQuery::new())
            .await
            .expect("sendable");
        println!("sendable: {}", templates.len());
        for t in &templates {
            assert!(t.is_approved(), "sendable but {:?}", t.status());
            println!("{:?} vars={}", t.status(), t.variable_names().len());
        }

        // `SOLAPI_PF_ID` is the channelId: filtering by it keeps only that channel.
        if let Ok(pf_id) = std::env::var("SOLAPI_PF_ID").map(|v| v.trim().to_owned())
            && !pf_id.is_empty()
        {
            let own = api()
                .sendable_templates(&SendableTemplateQuery::new().channel_id(&pf_id))
                .await
                .expect("sendable by channel");
            assert!(own.iter().all(|t| t.channel_id() == Some(pf_id.as_str())));
        }
    }

    /// Read-only: fetches `SOLAPI_TEST_TEMPLATE_ID`, or the first sendable template when unset (skipped
    /// when there is none).
    /// `cargo test --features solapi -- --ignored test_template_get --nocapture`
    #[tokio::test]
    #[ignore = "calls the real Solapi API"]
    async fn test_template_get() {
        let api = api();
        let id = match std::env::var("SOLAPI_TEST_TEMPLATE_ID") {
            Ok(id) if !id.trim().is_empty() => id,
            _ => match api
                .sendable_templates(&SendableTemplateQuery::new())
                .await
                .expect("sendable")
                .first()
            {
                Some(first) => first.template_id().to_owned(),
                None => {
                    println!("no template to fetch, skipped");
                    return;
                }
            },
        };
        let template = api.template(id.trim()).await.expect("template");
        println!(
            "{:?} approved={} vars={:?} buttons={}",
            template.status(),
            template.is_approved(),
            template.variable_names().len(),
            template.buttons().len()
        );
    }
}
