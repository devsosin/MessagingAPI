#[tokio::test]
async fn test() {
    assert!(true);
}

use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use messaging::AligoAPI;
use messaging::clients::aligo::AligoMessaging;
use messaging::configs::aligo::AligoConfig;
use serde_json::json;

#[tokio::test]
async fn test_serde() {
    let value = json!({
        "test": 'Y'
    });

    println!("{value}");
}

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

#[tokio::test]
async fn test_email() {
    let email = Message::builder()
        .from(Mailbox::new(
            Some("NoBody".to_owned()),
            "nobody@domain.tld".parse().unwrap(),
        ))
        .reply_to(Mailbox::new(
            Some("Yuin".to_owned()),
            "yuin@domain.tld".parse().unwrap(),
        ))
        .to(Mailbox::new(
            Some("Hei".to_owned()),
            "hei@domain.tld".parse().unwrap(),
        ))
        .subject("Happy new year")
        .header(ContentType::TEXT_PLAIN)
        .body(String::from("Be happy!"))
        .unwrap();

    let creds = Credentials::new("smtp_username".to_owned(), "smtp_password".to_owned());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }
}
