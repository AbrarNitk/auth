use chrono::Utc;
use std::ops::Sub;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct OtpBucketItem {
    otp: u32,
    expiry_at: i64,
}

impl OtpBucketItem {
    pub fn new(otp: u32) -> Self {
        Self {
            otp,
            expiry_at: Utc::now().timestamp(),
        }
    }
}

#[derive(Debug)]
pub struct OtpBucket(Vec<OtpBucketItem>);

impl OtpBucket {
    pub fn new(value: serde_json::Value) -> Result<OtpBucket, serde_json::Error> {
        Ok(Self(serde_json::from_value(value)?))
    }

    pub fn to_value(self) -> Result<serde_json::Value, serde_json::Error> {
        Ok(serde_json::to_value(self.0)?)
    }

    pub fn append(mut self, value: OtpBucketItem) -> Self {
        self.0.push(value);
        self
    }

    pub fn filter_old(mut self) -> Self {
        let five_min_old_time = Utc::now().timestamp().sub(5 * 60);
        let entries_greater_than_five_minutes = self
            .0
            .into_iter()
            .filter(|x| x.expiry_at.gt(&five_min_old_time))
            .collect();
        Self(entries_greater_than_five_minutes)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SendOtpError {
    #[error("SendMailError: {}", _0)]
    SendMail(#[from] crate::communication::SendMailError),
    #[error("SerdeError: {}", _0)]
    Serde(#[from] serde_json::Error),
    #[error("DBError: {}", _0)]
    DBError(#[from] db::DBError),
    #[error("DBError: {}", _0)]
    OTPNotFound(String),
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

pub async fn resend_otp(email: &str, db_pool: db::pg::DbPool) -> Result<(), SendOtpError> {
    let db_otp = db::auth::get_otp(email, &db_pool)?.ok_or(SendOtpError::OTPNotFound(format!(
        "Not otp has entry found with email: {email}"
    )))?;

    if db_otp.status.eq("VERIFIED") {
        return Err(SendOtpError::OTPNotFound(format!(
            "Send otp first before resending it with {email}"
        )));
    }

    let new_otp = generate_otp();
    let otp_bucket = OtpBucket::new(db_otp.otp_bucket)?
        .filter_old()
        .append(OtpBucketItem::new(new_otp));
    db::auth::otp_update_bucket(db_otp.id, &otp_bucket.to_value()?, "RESENDING", &db_pool)?;
    // crate::communication::send_email(new_otp, email, username).await?;
    db::auth::otp_update_status(db_otp.id, "RESEND", &db_pool)?;
    Ok(())
}

pub async fn verify_otp(
    email: &str,
    otp: u32,
    db_pool: db::pg::DbPool,
) -> Result<(), SendOtpError> {
    let otp = db::auth::get_otp(email, &db_pool)?.ok_or(SendOtpError::OTPNotFound(format!(
        "Not otp has entry found with email: {email}"
    )))?;

    Ok(())
}
