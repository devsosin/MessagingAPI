//! Send outcomes against a local server that answers like (or unlike) the provider. No network.

use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

/// Answers one request with `answer` (a 200 JSON body), or never answers when it is `None`.
/// Returns the base url.
fn one_shot_server(answer: Option<&'static str>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let url = format!("http://{}", listener.local_addr().expect("addr"));
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .ok();
        let mut buf = [0u8; 4096];
        while matches!(stream.read(&mut buf), Ok(n) if n > 0) {}
        match answer {
            Some(body) => {
                let res = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(res.as_bytes()).ok();
            }
            None => thread::sleep(Duration::from_secs(3)),
        }
    });
    url
}

fn client(timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .expect("client")
}

#[cfg(feature = "solapi")]
mod solapi {
    use std::time::Duration;

    use messaging::{
        solapi::{Solapi, config::SolapiConfig, types::Alimtalk},
        types::{Outcome, UnknownReason},
    };

    use super::{client, one_shot_server};

    async fn send(answer: Option<&'static str>, timeout: Duration) -> Outcome<String> {
        let url = one_shot_server(answer);
        let config = SolapiConfig::new("key", "secret", "0200000000", "PF");
        let api = Solapi::with_client(config, client(timeout)).with_base_url(url);
        let talks = [Alimtalk::new("01000000000", "TPL", [("회원명", "테스트")])];
        match api.send_alimtalks(&talks).await.expect("send") {
            Outcome::Sent(receipt) => Outcome::Sent(receipt.group_id().to_owned()),
            Outcome::Failed(failure) => Outcome::Failed(failure),
            Outcome::Unknown(reason) => Outcome::Unknown(reason),
        }
    }

    const LONG: Duration = Duration::from_secs(5);

    #[tokio::test]
    async fn answer_with_id_and_group_id_is_sent() {
        let answer = r#"{"groupInfo":{"_id":"G9","groupId":"G9","count":{"registeredFailed":0,"registeredSuccess":1}},"failedMessageList":[]}"#;
        assert_eq!(send(Some(answer), LONG).await, Outcome::Sent("G9".into()));
    }

    #[tokio::test]
    async fn unreadable_answer_is_unknown_with_the_parse_error() {
        let Outcome::Unknown(UnknownReason::UnreadableAnswer(detail)) =
            send(Some("not json"), LONG).await
        else {
            panic!("expected an unreadable answer")
        };
        assert!(detail.contains("expected"), "detail: {detail}");
    }

    #[tokio::test]
    async fn answer_without_group_id_is_unknown() {
        let answer = r#"{"count":{"registeredFailed":0,"registeredSuccess":1}}"#;
        assert_eq!(
            send(Some(answer), LONG).await,
            Outcome::Unknown(UnknownReason::NoGroupId)
        );
    }

    #[tokio::test]
    async fn no_answer_in_time_is_unknown_transport() {
        let outcome = send(None, Duration::from_millis(500)).await;
        assert!(
            matches!(outcome, Outcome::Unknown(UnknownReason::Transport(_))),
            "{outcome:?}"
        );
    }
}

#[cfg(feature = "aligo")]
mod aligo {
    use std::time::Duration;

    use messaging::{
        aligo::{
            AligoAPI,
            config::AligoConfig,
            types::{TextMessage, TextType},
        },
        types::{Outcome, UnknownReason},
    };

    use super::{client, one_shot_server};

    #[tokio::test]
    async fn unreadable_answer_is_unknown_with_the_parse_error() {
        let url = one_shot_server(Some("<html>busy</html>"));
        let config = AligoConfig::new("user", "key", "0200000000");
        let api = AligoAPI::with_client(config, client(Duration::from_secs(5))).with_base_url(url);
        let texts = [TextMessage::new("01000000000", "안녕하세요")];
        let outcome = api.send_mass(&texts, TextType::Sms).await.expect("send");
        assert!(
            matches!(&outcome, Outcome::Unknown(UnknownReason::UnreadableAnswer(d)) if !d.is_empty()),
            "{outcome:?}"
        );
    }
}
