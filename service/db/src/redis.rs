type Pool = r2d2::Pool<r2d2_redis::RedisConnectionManager>;
static REDIS_POOL: once_cell::sync::OnceCell<Pool> = once_cell::sync::OnceCell::new();

pub fn init_redis_pool() {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL env not found");
    let connection_manager = r2d2_redis::RedisConnectionManager::new(redis_url).unwrap();
    REDIS_POOL
        .set(
            Pool::builder()
                .build(connection_manager)
                .expect("Error in creating redis connection pool"),
        )
        .expect("Error in setting the REDIS_POOL for postgres");
}
