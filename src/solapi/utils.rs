use uuid::Uuid;

pub fn get_uuid() -> String {
    Uuid::new_v4().simple().to_string()
}

#[tokio::test]
async fn test_uuid() {
    let uuid = get_uuid();

    // uuid1 파라미터 바꾸지 않을 경우 계속 같은값 생성됨.
    let uuid = Uuid::new_v4();
    println!("{}", uuid.simple());
    // ff96562f85604b809ef502ff928d36d0
    // 81acde5ea08b4435b53dedf381c5ed1b
}
