static EMAIL_TEMPLATE: &str = include_str!("email-template.html");

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

#[derive(thiserror::Error, Debug)]
pub enum SendMailError {
    #[error("InvalidHeaderValueError: {}", _0)]
    InvalidHeaderValue(#[from] hyper::header::InvalidHeaderValue),
    #[error("ReqwestError: {}", _0)]
    Reqwest(#[from] reqwest::Error),
    #[error("SerdeSerializeError: {}", _0)]
    Serde(#[from] serde_json::Error),
}

pub async fn send_email(otp: u32, to_email: &str, to_username: &str) -> Result<(), SendMailError> {
    let mut headers = hyper::HeaderMap::new();
    headers.insert(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        "api-key",
        hyper::header::HeaderValue::try_from(crate::BREVO_API_KEY.as_str())?,
    );

    let request = SendEmailReq {
        sender: EmailUser {
            email: "wilderbit.net@gmail.com".to_owned(),
            name: "Wilderbit".to_owned(),
        },
        to: vec![EmailUser {
            email: to_email.to_owned(),
            name: to_username.to_owned(),
        }],
        bcc: vec![EmailUser {
            email: "wilderbit.net@gmail.com".to_owned(),
            name: "Wilderbit".to_owned(),
        }],
        cc: vec![EmailUser {
            email: "wilderbit.net@gmail.com".to_owned(),
            name: "Wilderbit".to_owned(),
        }],
        html_content: EMAIL_TEMPLATE
            .replace("__USER_NAME__", to_username)
            .replace("__OTP__", otp.to_string().as_str()),
        subject: "🔒 [Hasinam]: Your One-Time Password (OTP) for Secure Access".to_string(),
        reply_to: None,
        tags: vec!["OTP".to_owned()],
    };

    let client = reqwest::Client::new();
    let payload = serde_json::to_string(&request)?;
    println!("payload: {}", payload);
    let response = client
        .post("https://api.brevo.com/v3/smtp/email")
        .headers(headers)
        .json(&request)
        .send()
        .await?;

    let body = response.text().await?;
    if response.status().is_success() {
        println!("Send Email Response Success Body: {}", body);
    } else {
        println!(
            "Send Email Error: status: {}, body: {}",
            response.status(),
            body
        );
    }

    Ok(())
}
