static EMAIL_TEMPLATE: &str = include_str!("email-template.html");

#[derive(thiserror::Error, Debug)]
pub enum SendMailError {}

#[derive(serde::Serialize)]
pub struct EmailUser {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "email")]
    pub email: String,
}

#[derive(serde::Serialize)]
pub struct SendEmailReq {
    #[serde(rename = "sender")]
    pub sender: EmailUser,
    #[serde(rename = "to")]
    pub to: Vec<EmailUser>,
    #[serde(rename = "bcc")]
    pub bcc: Vec<EmailUser>,
    #[serde(rename = "cc")]
    pub cc: Vec<EmailUser>,
    #[serde(rename = "htmlContent")]
    pub html_content: String,
    #[serde(rename = "subject")]
    pub subject: String,
    #[serde(rename = "replyTo")]
    pub reply_to: Option<EmailUser>,
    #[serde(rename = "tags")]
    pub tags: Vec<String>,
}

pub async fn send_email() -> Result<(), SendMailError> {
    let mut headers = hyper::HeaderMap::new();
    headers.insert(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::try_from("application/json").unwrap(),
    );
    Ok(())
}
