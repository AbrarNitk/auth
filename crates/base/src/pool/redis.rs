pub type RedisPool = bb8::Pool<bb8_redis::RedisConnectionManager>;

/// Redis Connection Pool Helper
///
/// This module provides utilities for creating and managing Redis connection pools
/// using bb8 for efficient connection management in multi-tenant applications.
use anyhow::Context;
use bb8::Pool;
use bb8_redis::{RedisConnectionManager, redis::AsyncCommands};

pub async fn create(
    settings: &base::ctx::settings::redis::RedisSettings,
) -> anyhow::Result<RedisPool> {
    let manager = RedisConnectionManager::new(settings.url.clone())
        .context("Failed to create Redis connection manager")?;

    let pool = Pool::builder()
        .max_size(settings.max_size.unwrap_or(20)) // Maximum number of connections in the pool
        .min_idle(settings.min_idle.unwrap_or(5)) // Minimum number of idle connections in the pool
        .connection_timeout(std::time::Duration::from_secs(
            settings.connection_timeout.unwrap_or(10),
        )) // timeout to return the connection from the pool
        .idle_timeout(Some(std::time::Duration::from_secs(
            settings.idle_timeout.unwrap_or(300),
        ))) // timeout for idle connections
        .build(manager)
        .await
        .context("Failed to create Redis connection pool")?;

    // Test the connection
    let mut conn = pool
        .get()
        .await
        .context("Failed to get initial Redis connection")?;

    let _: String = conn
        .ping()
        .await
        .context("Redis ping failed during pool initialization")?;

    Ok(pool.clone())
}
