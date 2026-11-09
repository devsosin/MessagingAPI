use lettre::message::header::ContentType;

/// Body format of an email.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyType {
    Text,
    Html,
}

impl BodyType {
    pub(super) fn content_type(self) -> ContentType {
        match self {
            Self::Text => ContentType::TEXT_PLAIN,
            Self::Html => ContentType::TEXT_HTML,
        }
    }
}

/// One email to one receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email {
    receiver_name: Option<String>,
    receiver_email: String,
    subject: String,
    body: String,
    body_type: BodyType,
}

impl Email {
    pub fn new(
        receiver_email: impl Into<String>,
        subject: impl Into<String>,
        body: impl Into<String>,
        body_type: BodyType,
    ) -> Self {
        Self {
            receiver_name: None,
            receiver_email: receiver_email.into(),
            subject: subject.into(),
            body: body.into(),
            body_type,
        }
    }

    /// Display name shown with the receiver address.
    pub fn receiver_name(mut self, name: impl Into<String>) -> Self {
        self.receiver_name = Some(name.into());
        self
    }

    pub(super) fn receiver(&self) -> (Option<&str>, &str) {
        (self.receiver_name.as_deref(), &self.receiver_email)
    }

    pub(super) fn subject(&self) -> &str {
        &self.subject
    }

    pub(super) fn body(&self) -> &str {
        &self.body
    }

    pub(super) fn body_type(&self) -> BodyType {
        self.body_type
    }
}
