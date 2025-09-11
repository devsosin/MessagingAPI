use lettre::message::Mailbox;

use crate::{ClientResult, email::types::ReceiverGetter};

use super::EmailSender;

pub trait EmailMessaging {
    // compile time
    // use of `async fn` in public traits is discouraged as auto trait bounds cannot be specified
    // you can suppress this lint if you plan to use the trait only in your own code,
    // or do not care about auto traits like `Send` on the `Future`
    fn send_email<R: ReceiverGetter>(
        &self,
        receiver: &R,
        subject: &str,
        content: &str,
        is_html: bool,
    ) -> impl Future<Output = ClientResult<()>>;
}

impl EmailMessaging for EmailSender {
    async fn send_email<R: ReceiverGetter>(
        &self,
        receiver: &R,
        subject: &str,
        content: &str,
        is_html: bool,
    ) -> ClientResult<()> {
        let receiver_mailbox = Mailbox::new(
            receiver.get_name().to_owned(),
            receiver.get_address().parse().unwrap(),
        );

        self.send(&receiver_mailbox, subject, is_html, content)
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
