use bb8_redis::redis::AsyncCommands;

#[derive(Debug, Clone)]
pub struct RedisCache {
    pub redis_pool: base::RedisPool,
    pub key_prefix: String,
}

impl RedisCache {
    pub fn new(redis_pool: base::RedisPool, key_prefix: &str) -> Self {
        Self {
            redis_pool,
            key_prefix: key_prefix.to_owned(),
        }
    }

    pub async fn store<V: serde::Serialize>(
        &self,
        key: &str,
        value: &V,
        ttl_secs: u64,
    ) -> Result<(), StateCacheError> {
        let key = self.generate_key(key);

        let value = serde_json::to_string(value).map_err(|e| StateCacheError::SerializeError(e))?;

        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| StateCacheError::ConnectionError(e))?;

        let _: () = conn
            .set_ex(key.as_str(), value, ttl_secs)
            .await
            .map_err(|e| StateCacheError::CacheSetError(e.to_string()))?;
        Ok(())
    }

    pub async fn retrieve<V: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<V>, StateCacheError> {
        let key = self.generate_key(key);
        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| StateCacheError::ConnectionError(e))?;

        let json_value: Option<String> = conn
            .get(key.as_str())
            .await
            .map_err(|e| StateCacheError::CacheGetError(e))?;

        match json_value {
            Some(data) => {
                let value: V = serde_json::from_str(&data)
                    .map_err(|e| StateCacheError::DeserializeError(e))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub async fn invalidate(&self, key: &str) -> Result<(), StateCacheError> {
        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| StateCacheError::ConnectionError(e))?;
        let key = self.generate_key(key);
        let _: () = conn
            .del(&key)
            .await
            .map_err(|err| StateCacheError::CacheInvalidate(err.to_string()))?;
        Ok(())
    }

    fn generate_key(&self, key_suffix: &str) -> String {
        format!("{}{}", self.key_prefix, key_suffix)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StateCacheError {
    #[error("SerializeError: {}", _0)]
    SerializeError(serde_json::Error),
    #[error("ConnectionError: {}", _0)]
    ConnectionError(#[from] bb8_redis::bb8::RunError<bb8_redis::redis::RedisError>),
    #[error("DeserializeError: {}", _0)]
    DeserializeError(serde_json::Error),
    #[error("CacheGetError: {}", _0)]
    CacheGetError(bb8_redis::redis::RedisError),
    #[error("CacheSetError: {}", _0)]
    CacheSetError(String),
    #[error("CacheInvalidateError: {}", _0)]
    CacheInvalidate(String),
}
