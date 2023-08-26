use diesel::prelude::*;

pub fn upsert_with_email(email: &str, pool: &crate::pg::DbPool) -> Result<i64, crate::DBError> {
    use crate::schema::authapp_user;
    let mut conn = pool
        .get()
        .map_err(|x| crate::DBError::PooledConnection(x.to_string()))?;

    let now = chrono::Utc::now();
    let id = diesel::insert_into(authapp_user::dsl::authapp_user)
        .values((
            authapp_user::dsl::email.eq(email),
            authapp_user::dsl::active.eq(true),
            authapp_user::dsl::created_on.eq(now),
            authapp_user::dsl::updated_on.eq(now),
        ))
        .on_conflict(authapp_user::dsl::email)
        .do_update()
        .set((
            authapp_user::dsl::updated_on.eq(now),
            authapp_user::dsl::last_login.eq(now),
        ))
        .returning(authapp_user::dsl::id)
        .get_result::<i64>(&mut conn)?;
    Ok(id)
}

pub fn create_token(
    user_id: i64,
    token: &str,
    pool: &crate::pg::DbPool,
) -> Result<(), crate::DBError> {
    use crate::schema::authapp_user_token;
    let mut conn = pool
        .get()
        .map_err(|x| crate::DBError::PooledConnection(x.to_string()))?;
    conn.transaction(|conn| {
        let now = chrono::Utc::now();
        diesel::update(
            authapp_user_token::dsl::authapp_user_token
                .filter(authapp_user_token::dsl::user_id.eq(user_id))
                .filter(authapp_user_token::dsl::active.eq(true)),
        )
        .set((
            authapp_user_token::dsl::active.eq(false),
            authapp_user_token::dsl::updated_on.eq(now),
        ))
        .execute(conn)?;

        diesel::insert_into(authapp_user_token::dsl::authapp_user_token)
            .values((
                authapp_user_token::dsl::user_id.eq(user_id),
                authapp_user_token::dsl::active.eq(true),
                authapp_user_token::dsl::token.eq(token),
                authapp_user_token::dsl::created_on.eq(now),
                authapp_user_token::dsl::updated_on.eq(now),
            ))
            .execute(conn)?;
        diesel::result::QueryResult::Ok(())
    })?;

    Ok(())
}
