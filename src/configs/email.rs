enum EmailServer {
    Naver,
    Gmail,
}

pub struct EmailConfig {
    email_server: EmailServer,
    email_username: String,
    email_password: String,
}
