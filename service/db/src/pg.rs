type Pool = r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>;
static DB_POOL: once_cell::sync::OnceCell<Pool> = once_cell::sync::OnceCell::new();

pub fn init_db_pool() {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env not found");
    let connection_manager = r2d2_postgres::PostgresConnectionManager::new(
        db_url.parse().unwrap(),
        r2d2_postgres::postgres::NoTls,
    );
    DB_POOL
        .set(
            Pool::builder()
                .build(connection_manager)
                .expect("Error in creating connection pool"),
        )
        .expect("Error in setting the DB_POOL for postgres");
}
