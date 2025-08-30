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
        let receivers = vec!["00012345678", "00023456789"];
        let template_id = "test_template";

        struct SampleVariable {}
        impl ToAlimtalkVariable for SampleVariable {
            fn to_map(&self) -> HashMap<String, String> {
                let mut variable = HashMap::new();

                variable.insert("#{회원명}".into(), "테스트".into());

                variable
            }
        }

        let variables = vec![SampleVariable {}, SampleVariable {}];

        let response = api
            .send_alimtalks(template_id, &receivers, &variables)
            .await;

        println!("{:?}", response);
    }
}

#[cfg(feature = "aligo")]
mod aligo_tests {
    use messaging::aligo::AligoAPI;
    use messaging::aligo::client::AligoMessaging;
    use messaging::aligo::config::AligoConfig;

    #[tokio::test]
    async fn test_aligo() {
        use dotenv::dotenv;
        dotenv().ok();

        let config = AligoConfig::from_env();
        let api = AligoAPI::new(config);
        let receiver_list = vec!["00012345678", "00023456789"];
        let message_list = vec!["Test 1", "Test 2"];

        let res = api.send_sms(&receiver_list, &message_list, "sms").await;
        println!("{:?}", res);

        let res = api.send_mms("01012345678", "Test 1", "https://png.pngtree.com/thumb_back/fh260/background/20230613/pngtree-small-white-rabbit-in-the-grass-image_2915502.jpg").await;
        println!("{:?}", res);
    }
}

#[cfg(feature = "email")]
mod email_tests {
    use messaging::email::{
        EmailSender,
        client::EmailMessaging,
        config::EmailConfig,
        types::{EmailTemplateLoader, ReceiverGetter, ToEmailVariable},
    };
    use std::collections::HashMap;
    use tokio::fs;

    #[tokio::test]
    async fn test_email() {
        use dotenv::dotenv;
        dotenv().ok();

        let config = EmailConfig::from_env();
        let mail_sender = EmailSender::new(config);

        struct MyReceiver {
            name: Option<String>,
            address: String,
        }

        impl ReceiverGetter for MyReceiver {
            fn get_name(&self) -> &Option<String> {
                &self.name
            }
            fn get_address(&self) -> &str {
                &self.address
            }
        }

        let to_info = MyReceiver {
            name: Some("test1".into()),
            address: "test1234@gmail.test".into(),
        };

        let subject = "this mail is sent from rust";

        struct SampleTemplate {}

        impl EmailTemplateLoader for SampleTemplate {
            async fn get_content(&self) -> Result<String, std::io::Error> {
                let content_bytes = fs::read("./test_template.html").await?;
                let content = String::from_utf8(content_bytes).unwrap();
                Ok(content)
            }

            fn is_html(&self) -> bool {
                true
            }
        }

        let my_template = SampleTemplate {};

        struct SampleVariable {
            variables: Vec<[String; 2]>,
        }

        impl ToEmailVariable for SampleVariable {
            fn to_map(&self) -> HashMap<String, String> {
                let mut temp = HashMap::new();

                self.variables.iter().for_each(|v| {
                    let k1 = &v[0];
                    let v1 = &v[1];
                    temp.insert(k1.into(), v1.into());
                });

                temp
            }
        }

        let variables = vec![["{{test_variable}}".into(), "test".into()]];
        let my_variable = SampleVariable { variables };

        // Send the email
        let result = mail_sender
            .send_email(&to_info, subject, &my_template, &my_variable)
            .await;
        println!("{:?}", result);
    }
}
