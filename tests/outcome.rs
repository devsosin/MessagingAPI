//! Send outcomes against a local server that answers like (or unlike) the provider. No network.

use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

/// Answers one request with `answer` (a 200 JSON body), or never answers when it is `None`.
/// Returns the base url.
fn one_shot_server(answer: Option<&'static str>) -> String {
    recording_server(answer).0
}

/// [`one_shot_server`] that also hands over the request line (`POST /path HTTP/1.1`).
fn recording_server(answer: Option<&'static str>) -> (String, mpsc::Receiver<String>) {
    recording_server_with("200 OK", answer)
}

/// [`recording_server`] answering with another status line (`"401 Unauthorized"`).
fn recording_server_with(
    status: &'static str,
    answer: Option<&'static str>,
) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let url = format!("http://{}", listener.local_addr().expect("addr"));
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .ok();
        let mut request = Vec::new();
        let mut buf = [0u8; 4096];
        while let Ok(n) = stream.read(&mut buf) {
            if n == 0 {
                break;
            }
            request.extend_from_slice(&buf[..n]);
        }
        let head = String::from_utf8_lossy(&request);
        tx.send(head.lines().next().unwrap_or_default().to_owned())
            .ok();
        match answer {
            Some(body) => {
                let res = format!(
                    "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(res.as_bytes()).ok();
            }
            None => thread::sleep(Duration::from_secs(3)),
        }
    });
    (url, rx)
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
        solapi::{
            Solapi,
            config::SolapiConfig,
            types::{Alimtalk, SendableTemplateQuery, TemplateStatus},
        },
        types::{FailureKind, Outcome, UnknownReason},
    };

    use super::{client, one_shot_server, recording_server, recording_server_with};

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
    async fn alimtalks_go_to_send_many_detail() {
        let answer = r#"{"groupInfo":{"_id":"G9","groupId":"G9","count":{"registeredFailed":0,"registeredSuccess":1}},"failedMessageList":[]}"#;
        let (url, request) = recording_server(Some(answer));
        let config = SolapiConfig::new("key", "secret", "0200000000", "PF");
        let api = Solapi::with_client(config, client(LONG)).with_base_url(url);
        let talks = [Alimtalk::new("01000000000", "TPL", [("회원명", "테스트")])];
        api.send_alimtalks(&talks).await.expect("send");
        assert_eq!(
            request.recv().expect("request line"),
            "POST /messages/v4/send-many/detail HTTP/1.1"
        );
    }

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

    fn template_api(url: String) -> Solapi {
        let config = SolapiConfig::new("key", "secret", "0200000000", "PF");
        Solapi::with_client(config, client(LONG)).with_base_url(url)
    }

    const FULL_TEMPLATE: &str = r##"{
        "templateId": "TPL1", "code": "welcome", "name": "가입 안내", "channelId": "CH1",
        "channelGroupId": "G", "assignType": "CHANNEL", "accountId": "A", "securityFlag": false,
        "categoryCode": "001001", "isHidden": false, "isDeleted": false,
        "content": "#{회원명}님 #{코드} 안내", "variables": [{"name": "회원명"}, {"name": "#{코드}"}],
        "buttons": [{"buttonType": "NEWTYPE", "buttonName": "열기", "linkMo": "https://m.example",
                     "linkPc": "https://example", "targetOut": true}],
        "quickReplies": [], "status": "APPROVED", "messageType": "BA", "emphasizeType": "NONE",
        "comments": [], "futureField": 1,
        "dateCreated": "2026-01-01T00:00:00.000Z", "dateUpdated": "2026-01-02T00:00:00.000Z"
    }"##;

    #[tokio::test]
    async fn sendable_templates_parse_full_and_sparse_items() {
        let answer: &'static str =
            Box::leak(format!(r#"[{FULL_TEMPLATE}, {{"templateId": "T2"}}]"#).into_boxed_str());
        let (url, request) = recording_server(Some(answer));
        let list = template_api(url)
            .sendable_templates(&SendableTemplateQuery::new())
            .await
            .expect("list");
        assert_eq!(
            request.recv().expect("request line"),
            "GET /kakao/v2/templates/sendable HTTP/1.1"
        );
        assert_eq!(list.len(), 2);
        let full = &list[0];
        assert_eq!(full.template_id(), "TPL1");
        assert_eq!(full.code(), Some("welcome"));
        assert_eq!(full.name(), "가입 안내");
        assert_eq!(full.channel_id(), Some("CH1"));
        assert!(full.is_approved());
        assert_eq!(full.content(), "#{회원명}님 #{코드} 안내");
        assert_eq!(full.variable_names(), ["회원명", "코드"]);
        assert_eq!(full.buttons()[0].button_type(), Some("NEWTYPE"));
        assert_eq!(full.buttons()[0].link_mobile(), Some("https://m.example"));
        let sparse = &list[1];
        assert_eq!(sparse.template_id(), "T2");
        assert_eq!(sparse.code(), None);
        assert!(sparse.variable_names().is_empty());
        assert!(!sparse.is_approved());
    }

    #[tokio::test]
    async fn sendable_templates_accept_an_empty_list() {
        let list = template_api(one_shot_server(Some("[]")))
            .sendable_templates(&SendableTemplateQuery::new())
            .await
            .expect("list");
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn sendable_query_is_percent_encoded() {
        let (url, request) = recording_server(Some("[]"));
        let query = SendableTemplateQuery::new()
            .channel_id("CH 1&x=y")
            .template_code("가")
            .sender_key("");
        template_api(url)
            .sendable_templates(&query)
            .await
            .expect("list");
        assert_eq!(
            request.recv().expect("request line"),
            "GET /kakao/v2/templates/sendable?channelId=CH%201%26x%3Dy&templateCode=%EA%B0%80 HTTP/1.1"
        );
    }

    #[tokio::test]
    async fn template_is_fetched_by_filtered_id() {
        let answer: &'static str = Box::leak(FULL_TEMPLATE.to_owned().into_boxed_str());
        let (url, request) = recording_server(Some(answer));
        let template = template_api(url)
            .template("KA01/../TP-1?x=1")
            .await
            .expect("template");
        assert_eq!(
            request.recv().expect("request line"),
            "GET /kakao/v2/templates/KA01TP-1x1 HTTP/1.1"
        );
        assert_eq!(template.template_id(), "TPL1");
    }

    #[tokio::test]
    async fn unknown_and_dormant_statuses_are_kept() {
        let answer = r#"[{"templateId":"A","status":"DORMANT"},{"templateId":"B","status":"SOMETHING_NEW"},{"templateId":"C","status":"INSPECTING"}]"#;
        let list = template_api(one_shot_server(Some(answer)))
            .sendable_templates(&SendableTemplateQuery::new())
            .await
            .expect("list");
        assert_eq!(list[0].status(), &TemplateStatus::Dormant);
        assert_eq!(
            list[1].status(),
            &TemplateStatus::Other("SOMETHING_NEW".into())
        );
        assert_eq!(list[2].status(), &TemplateStatus::Inspecting);
        assert!(list.iter().all(|t| !t.is_approved()));
    }

    #[tokio::test]
    async fn variables_fall_back_to_the_content_placeholders() {
        let answer = r##"{"templateId":"A","content":"#{a} #{b} #{a} #{"}"##;
        let template = template_api(one_shot_server(Some(answer)))
            .template("A")
            .await
            .expect("template");
        assert_eq!(template.variable_names(), ["a", "b"]);
    }

    #[tokio::test]
    async fn http_errors_become_failures() {
        let body = r#"{"errorCode":"InvalidApiKey","errorMessage":"bad key"}"#;
        let (url, _request) = recording_server_with("401 Unauthorized", Some(body));
        let failure = template_api(url).template("A").await.expect_err("failure");
        assert_eq!(failure.kind(), FailureKind::AuthFailed);

        let (url, _request) = recording_server_with("500 Internal Server Error", Some("oops"));
        let failure = template_api(url)
            .sendable_templates(&SendableTemplateQuery::new())
            .await
            .expect_err("failure");
        assert_ne!(failure.kind(), FailureKind::AuthFailed);
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
