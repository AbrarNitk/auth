#[derive(serde::Serialize, serde::Deserialize)]
struct OtpBucketItem {
    otp: u32,
    expiry_at: i64,
}

#[derive(thiserror::Error, Debug)]
pub enum SendOtpError {
    #[error("SendMailError: {}", _0)]
    SendMail(#[from] crate::communication::SendMailError),
    #[error("SerdeError: {}", _0)]
    Serde(#[from] serde_json::Error),
    #[error("DBError: {}", _0)]
    DBError(#[from] db::DBError),
}

fn generate_otp() -> u32 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(100000..=999_999)
}

pub async fn send_otp(
    email: &str,
    username: &str,
    db_pool: db::pg::DbPool,
) -> Result<(), SendOtpError> {
    let otp = generate_otp();

    let otp_bucket = vec![OtpBucketItem {
        otp,
        expiry_at: chrono::Utc::now().timestamp(),
    }];
    let otp_id = db::auth::otp_upsert(
        email,
        &serde_json::to_value(&otp_bucket)?,
        "SENDING",
        &db_pool,
    )?;
    // crate::communication::send_email(otp, email, username).await?;
    db::auth::otp_update_status(otp_id, "SEND", &db_pool)?;
    Ok(())
}

pub async fn resend_otp(email: &str, username: &str) -> Result<(), SendOtpError> {
    let otp = generate_otp();
    crate::communication::send_email(otp, email, username).await?;
    Ok(())
}

pub async fn verify_otp(email: &str, username: &str) -> Result<(), SendOtpError> {
    // get the otp and verify if otp is not expired
    Ok(())
}
