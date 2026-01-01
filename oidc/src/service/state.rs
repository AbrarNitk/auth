use hmac::{Hmac, Mac};
use openidconnect::{CsrfToken, PkceCodeChallenge};
use sha2::Sha256;
pub use std::time::{SystemTime, UNIX_EPOCH};
type HmacSha256 = Hmac<Sha256>;
// use authnz::infra::cache::{RedisCache, StateCacheError};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthState {
    // OIDC partner name
    pub oidc_partner: String,

    // User Session ID(ULID)
    pub user_session_id: String,

    // Nonce for ID Token validation(Prevent replay attacks)
    pub nonce: String,

    // Code Verifier for PKCE(Proof Key for Code Exchange)
    pub code_verifier: String,

    // Return URL(Redirect URL after authentication is done)
    pub return_url: Option<String>,

    // Timestamp when the state was created(Unix epoch seconds)
    pub created_at: u64,

    // CSRF Token (for validation during callback)
    pub csrf_token: Option<String>,

    // Timestamp when the state will expire(Unix epoch seconds)
    pub expires_at: u64,

    // Client IP Address (for validation during callback)
    // We generally do not throw error if this is not present, but we warn in the system logs
    pub ip_address: Option<String>,

    // Client User Agent (for validation during callback)
    // We generally do not throw error if this is not present, but we warn in the system logs
    // We use the hash of the user-agent to prevent the user-agent fingerprinting
    pub user_agent_hash: Option<String>,
}

impl AuthState {
    pub fn new(
        oidc_partner: &str,
        return_url: &Option<String>,
        ttl_seconds: u64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expires_at = created_at + ttl_seconds;

        let (_, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let nonce = CsrfToken::new_random(); // Using CsrfToken for nonce generation
        let csrf_token = CsrfToken::new_random();

        Self {
            oidc_partner: oidc_partner.to_owned(),
            user_session_id: ulid::Ulid::new().to_string(),
            nonce: nonce.secret().to_string(),
            code_verifier: pkce_verifier.secret().to_string(),
            return_url: return_url.to_owned(),
            created_at: created_at,
            expires_at: expires_at,
            csrf_token: Some(csrf_token.secret().to_string()),
            ip_address: ip_address,
            user_agent_hash: user_agent.map(|agent| hash_user_agent(&agent)),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }
}

/// Management of AuthState: store, retrieve and invalidate the state
pub struct AuthStateManager {
    state_cache: crate::db::cache::RedisCache,
}

impl AuthStateManager {
    pub fn new(redis_pool: base::RedisPool) -> Self {
        Self {
            state_cache: crate::db::cache::RedisCache {
                redis_pool,
                key_prefix: "auth-login-state".to_owned(),
            },
        }
    }

    pub async fn store(&self, state: &AuthState) -> Result<(), AuthStateError> {
        self.state_cache
            .store(
                &state.user_session_id,
                state,
                state.expires_at - state.created_at,
            )
            .await?;
        Ok(())
    }

    // validating the auth-state
    // match the stored ip and user-agent but do not throw error, if mismatch
    // if time is expired, or org-id mismatch, we return the false
    #[tracing::instrument(name = "auth-state-validation", skip_all)]
    pub fn validate(
        &self,
        state: &AuthState,
        client_ip: &Option<String>,
        client_user_agent: &Option<String>,
    ) -> bool {
        tracing::info!(
            msg = "validate-auth-state",
            state_id = state.user_session_id
        );

        if state.is_expired() {
            tracing::error!(msg = "state-expired");
            return false;
        }

        if !state.ip_address.eq(client_ip) {
            tracing::warn!(
                msg = "client-ip-address-mismatch",
                before = state.ip_address,
                after = client_ip
            );
        }

        let ua = client_user_agent.as_ref().map(|ua| hash_user_agent(ua));
        if !ua.eq(&state.user_agent_hash) {
            tracing::warn!(
                msg = "client-user-agent-mismatch",
                hash_before = state.user_agent_hash,
                hash_after = ua,
                value_after = client_user_agent
            );
        }
        true
    }

    pub async fn retrieve_auth_state(
        &self,
        session_state_id: &str,
    ) -> Result<Option<AuthState>, AuthStateError> {
        Ok(self.state_cache.retrieve(session_state_id).await?)
    }

    pub async fn invalidate_auth_state(
        &self,
        session_state_id: &str,
    ) -> Result<(), AuthStateError> {
        self.state_cache.invalidate(session_state_id).await?;
        Ok(())
    }
}

/// Signed AuthState
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SignedState {
    // Unique State Identifier(used in the redis)
    pub state_id: String,

    // Timestamp when the state was signed(Unix epoch seconds)
    pub timestamp: u64,

    // Signature of the state
    pub signature: String,
}

impl SignedState {
    pub fn new(state_id: &str, secret: &str) -> Result<Self, AuthStateError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let signature = Self::compute_signature(state_id, timestamp, secret)?;
        Ok(Self {
            state_id: state_id.to_owned(),
            timestamp: timestamp,
            signature: signature,
        })
    }

    fn compute_signature(
        state_id: &str,
        timestamp: u64,
        secret: &str,
    ) -> Result<String, AuthStateError> {
        let mut hasher = HmacSha256::new_from_slice(secret.as_bytes()).map_err(|err| {
            AuthStateError::SecretHMacError(format!("computing sha for the secret failed: {}", err))
        })?; //todo: handle error

        hasher.update(state_id.as_bytes());
        hasher.update(&timestamp.to_le_bytes());
        Ok(hex::encode(hasher.finalize().into_bytes()))
    }

    pub fn encode(&self) -> Result<String, AuthStateError> {
        let data = serde_json::to_string(self).map_err(|e| AuthStateError::SerializeError(e))?;
        Ok(URL_SAFE_NO_PAD.encode(&data))
    }

    pub fn decode(encoded: &str, secret: &str) -> Result<Self, AuthStateError> {
        let data = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|e| AuthStateError::DecodeError(e.to_string()))?;

        let state: Self = serde_json::from_slice(data.as_slice())
            .map_err(|e| AuthStateError::DeserializeError(e))?;

        state.verify_signature(secret)?;
        Ok(state)
    }

    fn verify_signature(&self, secret: &str) -> Result<(), AuthStateError> {
        let computed_signature =
            Self::compute_signature(self.state_id.as_str(), self.timestamp, secret)?;

        if computed_signature != self.signature {
            return Err(AuthStateError::SignatureMismatchError(
                "signature verification failed".to_string(),
            ));
        }
        Ok(())
    }
}

/// Hash user agent for privacy and validation
fn hash_user_agent(user_agent: &str) -> String {
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(user_agent.as_bytes());
    hex::encode(hasher.finalize())
}

#[derive(Debug, thiserror::Error)]
pub enum AuthStateError {
    #[error("SecretHMacError: {}", _0)]
    SecretHMacError(String),
    #[error("SignatureMismatchError: {}", _0)]
    SignatureMismatchError(String),
    #[error("DecodeError: {}", _0)]
    DecodeError(String),
    #[error("DeserializeError: {}", _0)]
    DeserializeError(serde_json::Error),
    #[error("SerializeError: {}", _0)]
    SerializeError(serde_json::Error),
    #[error("EncodeError: {}", _0)]
    EncodeError(String),
    #[error("AuthStateCacheError: {}", _0)]
    AuthStateCacheError(#[from] crate::db::cache::StateCacheError),
}
