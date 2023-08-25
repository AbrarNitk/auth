#[derive(thiserror::Error, Debug)]
pub enum SendOtpError {
    #[error("SendMailError: {}", _0)]
    SendMail(#[from] crate::communication::SendMailError),
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
    // sending the email
    crate::communication::send_email(otp, email, username).await?;
    // send email
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
