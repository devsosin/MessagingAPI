# Messaing API (Send Email, Kakao Talk, SMS)

## Providers

- ### Eamil Providers
    > #### Mail Servers
    > - gmail
    > - naver

- ### Solapi
    > #### Key Point
    > - 발신번호는 사전 등록 필수 → 발신번호 관리
    > - 발송 내역 확인 → 문자발송 내역
    > - 데이터 보관기간: 6개월 (생성일 기준)
    > - 발송할 수 있는 메시지가 없을 경우 자세한 에러 내역을 결과로 반환

- ### Aligo

## Environment Variables

**ㅁ for sending email**

```
EMAIL_SERVER=
EMAIL_USERNAME=
EMAIL_SENDER_NAME=
EMAIL_PASSWORD=
```



**ㅁ for send Alimtalk (Solapi)**

```
SOLAPI_API_KEY=
SOLAPI_API_SECRET=
SOLAPI_PF_ID=
SOLAPI_SENDER_NUMBER=
```


**ㅁ for sending sms (ALigo)**

```
ALIGO_API_KEY=
ALIGO_USER_ID=
ALIGO_SENDER_NUMBER=
ALIGO_TEST=Y/N
```

## Run Examples

### Email

```rs
async fn test_email() {
    let mail_sender = EmailConfig::from_env().to_sender();

    let subject = "this mail is sent from rust";
    let content = "";

    // Send the email
    let result = mail_sender
        .send_email(
            &Some("test1".into()), // receiver name (optional)
            "test1234@gmail.com", // receiver address
            subject,
            content,
            false, // is_html
        )
        .await;
    println!("{:?}", result);
}
```

### Solapi

```rs
async fn test_solapi() {
    let solapi = SolapiConfig::from_env().to_sender();
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
```

### Aligo

```rs
async fn test_aligo() {
    let api = AligoConfig::from_env().to_sender();
    let receiver_list = vec!["00012345678", "00023456789"];
    let message_list = vec!["Test 1", "Test 2"];

    let res = api.send_sms(&receiver_list, &message_list, "sms").await;
    println!("{:?}", res);

    let res = api.send_mms("01012345678", "Test 1", "https://png.pngtree.com/thumb_back/fh260/background/20230613/pngtree-small-white-rabbit-in-the-grass-image_2915502.jpg").await;
    println!("{:?}", res);
}
```