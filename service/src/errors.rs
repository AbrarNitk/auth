#[derive(thiserror::Error, Debug)]
pub enum RouteError {
    #[error("JsonSerializeError: {0}")]
    JsonSerializeError(#[from] serde_json::Error),
    #[error("GetProfileError: {0}")]
    GetProfileError(#[from] http_service::controller::GetProfileError),
    #[error("AuthError")]
    AuthError(#[from] auth::error::AuthError),
    #[error("FileReadError: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("AIError")]
    AIError(#[from] ai::apis::AIError),
}
