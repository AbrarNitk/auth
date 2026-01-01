pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub fn get_connection_pool(url: &str) -> DbPool {
    let connection_manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(url);
    diesel::r2d2::Pool::builder()
        .max_size(10)
        .idle_timeout(Some(std::time::Duration::from_secs(600)))
        .connection_timeout(std::time::Duration::from_secs(30))
        .build(connection_manager)
        .expect("Error in building the connection pool for postgres")
}

pub async fn pool_with_url(url: &str) -> anyhow::Result<sqlx::postgres::PgPool> {
    let opts: sqlx::postgres::PgConnectOptions = url.parse()?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .min_connections(3)
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(20))
        // connection will be timed out after this time, if it site idle
        .idle_timeout(Some(std::time::Duration::from_secs(10 * 60)))
        .max_lifetime(Some(std::time::Duration::from_secs(30 * 60)))
        .test_before_acquire(true)
        .connect_with(opts)
        .await?;
    Ok(pool)
}
