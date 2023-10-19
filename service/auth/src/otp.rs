#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct OtpBucketItem {
    otp: u32,
    expiry_at: i64,
}

impl OtpBucketItem {
    pub fn new(otp: u32) -> Self {
        Self {
            otp,
            expiry_at: chrono::Utc::now().timestamp(),
        }
    }

    fn is_item_older_than_5_mins(&self) -> bool {
        use std::ops::Sub;
        let five_min_old_time = chrono::Utc::now().timestamp().sub(5 * 60);
        self.expiry_at.gt(&five_min_old_time)
    }
}

#[derive(Debug)]
pub struct OtpBucket(Vec<OtpBucketItem>);

impl OtpBucket {
    pub fn new(value: serde_json::Value) -> Result<OtpBucket, serde_json::Error> {
        Ok(Self(serde_json::from_value(value)?))
    }

    pub fn to_value(self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self.0)
    }

    pub fn append(mut self, value: OtpBucketItem) -> Self {
        self.0.push(value);
        self
    }

    pub fn filter_old(self) -> Self {
        let items_greater_than_five_minutes = self
            .0
            .into_iter()
            .filter(OtpBucketItem::is_item_older_than_5_mins)
            .collect();
        Self(items_greater_than_five_minutes)
    }

    pub fn verify_otp(&self, otp: u32) -> bool {
        self.0
            .iter()
            .filter(|x| OtpBucketItem::is_item_older_than_5_mins(x))
            .any(|x| x.otp.eq(&otp))
    }

    pub fn empty(self) -> Self {
        Self(vec![])
    }
}

#[derive(thiserror::Error, Debug)]
pub enum OtpError {
    #[error("SendMailError: {}", _0)]
    SendMail(#[from] crate::communication::SendMailError),
    #[error("SerdeError: {}", _0)]
    Serde(#[from] serde_json::Error),
    #[error("DBError: {}", _0)]
    DBError(#[from] db::DBError),
    #[error("OTPNotFound: {}", _0)]
    OTPNotFound(String),
    #[error("OTPExpired: {}", _0)]
    Expired(String),
    #[error("AmbiguousVerificationRequest: {}", _0)]
    AmbiguousVerificationRequest(String),
    #[error("JWTError: {}", _0)]
    JWT(#[from] crate::jwt::JWTError),
}

fn generate_otp() -> u32 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(100000..=999_999)
}

#[derive(serde::Deserialize)]
pub struct SendOtpReq {
    pub email: String,
    pub phone: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SendOtpRes {
    pub email: String,
    pub message: String,
}

pub async fn send_otp(
    otp_req: SendOtpReq,
    db_pool: db::pg::DbPool,
) -> Result<SendOtpRes, OtpError> {
    let otp = generate_otp();
    let otp_bucket = vec![OtpBucketItem {
        otp,
        expiry_at: chrono::Utc::now().timestamp(),
    }];
    let otp_id = db::otp::otp_upsert(
        otp_req.email.as_str(),
        &serde_json::to_value(otp_bucket)?,
        "SENDING",
        &db_pool,
    )?;
    crate::communication::send_email(otp, otp_req.email.as_str()).await?;
    db::otp::otp_update_status(otp_id, "SEND", &db_pool)?;
    Ok(SendOtpRes {
        email: otp_req.email,
        message: "OTP send successfully".to_string(),
    })
}

pub async fn resend_otp(
    otp_req: SendOtpReq,
    db_pool: db::pg::DbPool,
) -> Result<SendOtpRes, OtpError> {
    let db_otp =
        db::otp::get_otp(otp_req.email.as_str(), &db_pool)?.ok_or(OtpError::OTPNotFound(
            format!("Not otp has entry found with email: {}", otp_req.email),
        ))?;

    if db_otp.status.eq("VERIFIED") {
        return Err(OtpError::OTPNotFound(format!(
            "Send otp first before resending it with {}",
            otp_req.email
        )));
    }

    let new_otp = generate_otp();
    let otp_bucket = OtpBucket::new(db_otp.otp_bucket)?
        .filter_old()
        .append(OtpBucketItem::new(new_otp));
    db::otp::otp_update_bucket(db_otp.id, &otp_bucket.to_value()?, "RESENDING", &db_pool)?;
    crate::communication::send_email(new_otp, otp_req.email.as_str()).await?;
    db::otp::otp_update_status(db_otp.id, "RESEND", &db_pool)?;
    Ok(SendOtpRes {
        email: otp_req.email,
        message: "OTP resend successfully".to_string(),
    })
}

#[derive(serde::Deserialize)]
pub struct VerifyOtpReq {
    #[serde(rename = "email")]
    pub email: String,
    #[serde(rename = "phone")]
    pub phone: Option<String>,
    pub otp: u32,
}

#[derive(serde::Serialize)]
pub struct VerifyOtpRes {
    user_token: String,
}

pub async fn verify_otp(
    otp_req: VerifyOtpReq,
    db_pool: db::pg::DbPool,
) -> Result<VerifyOtpRes, OtpError> {
    let db_otp =
        db::otp::get_otp(otp_req.email.as_str(), &db_pool)?.ok_or(OtpError::OTPNotFound(
            format!("Not otp has entry found with email: {}", otp_req.email),
        ))?;

    if db_otp.status.eq("VERIFIED") {
        return Err(OtpError::AmbiguousVerificationRequest(
            "otp is already expired".to_string(),
        ));
    }

    let otp_bucket = OtpBucket::new(db_otp.otp_bucket)?;
    if !otp_bucket.verify_otp(otp_req.otp) {
        return Err(OtpError::Expired(
            "OTP is expired resend the otp again".to_string(),
        ));
    }

    tracing::info!(message="otp is verified", email=otp_req.email);
    // get or create user
    let user_id = db::user::upsert_with_email(otp_req.email.as_str(), &db_pool)?;
    // generate the token
    let jwt_token = crate::jwt::create_jwt(user_id.to_string())?;
    // inactive all the active tokens if any and issue the new token
    db::user::create_token(user_id, jwt_token.as_str(), &db_pool)?;
    db::otp::otp_update_bucket(
        db_otp.id,
        &otp_bucket.empty().to_value()?,
        "VERIFIED",
        &db_pool,
    )?;

    Ok(VerifyOtpRes {
        user_token: jwt_token,
    })
}
