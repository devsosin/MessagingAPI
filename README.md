# Messaing API (Send Email, Kakao Talk, SMS)

## Providers

- Eamil Providers
- Solapi
- Aligo

## Environment Variables

**ㅁ for sending email**

```
EMAIL_SERVER=
EMAIL_USERNAME=
EMAIL_PASSWORD=
```

**ㅁ for send Alimtalk (Solapi)**

```
SOLAPI_API_KEY=
SOLAPI_API_SECRET=
SOLAPI_PF_ID=
SOLAPI_SENDER_NUMBER=
```

> ### 주요 사항
> - 발신번호는 사전 등록 필수 → 발신번호 관리
> - 발송 내역 확인 → 문자발송 내역
> - 데이터 보관기간: 6개월 (생성일 기준)
> - 발송할 수 있는 메시지가 없을 경우 자세한 에러 내역을 결과로 반환

**ㅁ for sending sms (ALigo)**

```
ALIGO_API_KEY=
ALIGO_USER_ID=
ALIGO_SENDER_NUMBER=
ALIGO_TEST=Y/N
```

