use diesel::prelude::*;
use diesel::{OptionalExtension, RunQueryDsl};

#[derive(diesel::Queryable)]
pub struct OtpDB {
    pub id: i64,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub otp_bucket: serde_json::Value,
    pub status: String,
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub updated_on: chrono::DateTime<chrono::Utc>,
}

pub fn get_otp(
    user_email: &str,
    db_pool: crate::pg::DbPool,
) -> Result<Option<OtpDB>, diesel::result::Error> {
    use crate::schema::authapp_user_otp;
    let mut conn = db_pool.get().unwrap();
    Ok(authapp_user_otp::dsl::authapp_user_otp
        .filter(authapp_user_otp::dsl::email.eq(user_email))
        .select((
            authapp_user_otp::dsl::id,
            authapp_user_otp::dsl::email,
            authapp_user_otp::dsl::phone,
            authapp_user_otp::dsl::otp_bucket,
            authapp_user_otp::dsl::status,
            authapp_user_otp::dsl::created_on,
            authapp_user_otp::dsl::updated_on,
        ))
        .get_result::<OtpDB>(&mut conn)
        .optional()?)
}
