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
) -> Result<Option<OtpDB>, crate::DBError> {
    use crate::schema::authapp_user_otp;
    let mut conn = db_pool
        .get()
        .map_err(|x| crate::DBError::PooledConnection(x.to_string()))?;
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

pub fn otp_upsert(
    email: &str,
    otp: &serde_json::Value,
    status: &str,
    db_pool: &crate::pg::DbPool,
) -> Result<i64, crate::DBError> {
    use crate::schema::authapp_user_otp;
    let mut conn = db_pool
        .get()
        .map_err(|x| crate::DBError::PooledConnection(x.to_string()))?;
    Ok(diesel::insert_into(authapp_user_otp::dsl::authapp_user_otp)
        .values((
            authapp_user_otp::dsl::email.eq(email),
            authapp_user_otp::dsl::otp_bucket.eq(otp),
            authapp_user_otp::dsl::status.eq(status),
            authapp_user_otp::dsl::created_on.eq(chrono::Utc::now()),
            authapp_user_otp::dsl::updated_on.eq(chrono::Utc::now()),
        ))
        .on_conflict(authapp_user_otp::dsl::email)
        .do_update()
        .set((
            authapp_user_otp::dsl::otp_bucket.eq(otp),
            authapp_user_otp::dsl::status.eq(status),
            authapp_user_otp::dsl::updated_on.eq(chrono::Utc::now()),
        ))
        .returning(authapp_user_otp::dsl::id)
        .get_result::<i64>(&mut conn)?)
}

pub fn otp_update_status(
    id: i64,
    status: &str,
    pool: &crate::pg::DbPool,
) -> Result<(), crate::DBError> {
    use crate::schema::authapp_user_otp;
    let mut conn = pool
        .get()
        .map_err(|x| crate::DBError::PooledConnection(x.to_string()))?;
    diesel::update(
        authapp_user_otp::dsl::authapp_user_otp.filter(authapp_user_otp::dsl::id.eq(id)),
    )
    .set((
        authapp_user_otp::dsl::status.eq(status),
        authapp_user_otp::dsl::updated_on.eq(chrono::Utc::now()),
    ))
    .execute(&mut conn)?;
    Ok(())
}
