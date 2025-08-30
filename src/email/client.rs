use lettre::message::Mailbox;

use crate::{
    ClientResult,
    email::types::{EmailTemplateLoader, ReceiverGetter, ToEmailVariable},
    errors::ClientError,
};

use super::EmailSender;

pub trait EmailMessaging {
    // compile time
    // use of `async fn` in public traits is discouraged as auto trait bounds cannot be specified
    // you can suppress this lint if you plan to use the trait only in your own code,
    // or do not care about auto traits like `Send` on the `Future`
    fn send_email<T: EmailTemplateLoader, R: ReceiverGetter, V: ToEmailVariable>(
        &self,
        receiver: &R,
        subject: &str,
        template: &T,
        variable: &V,
    ) -> impl Future<Output = ClientResult<()>>;
}

impl EmailMessaging for EmailSender {
    async fn send_email<T: EmailTemplateLoader, R: ReceiverGetter, V: ToEmailVariable>(
        &self,
        receiver: &R,
        subject: &str,
        template: &T,
        variable: &V,
    ) -> ClientResult<()> {
        let receiver_mailbox = Mailbox::new(
            receiver.get_name().to_owned(),
            receiver.get_address().parse().unwrap(),
        );
        let content = template.get_content().await.map_err(|e| {
            println!("{:?}", e);
            ClientError::InternalError("Template Load Failed")
        })?;

        let content = variable
            .to_map()
            .iter()
            .fold(content, |c, (k, v)| c.replace(k, v));

        self.send(&receiver_mailbox, subject, template.is_html(), &content)
            .await
    }
}

#[tokio::test]
async fn test_fold() {
    use std::collections::HashMap;

    let content = "This is for {{test_variable}}".to_string();

    let mut my_map = HashMap::new();
    my_map.insert("{{test_variable}}", "testing");

    let result = my_map.iter().fold(content, |c, (k, v)| c.replace(k, v));

    println!("{}", result);
}
