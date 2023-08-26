#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("BodyReadError: {0}")]
    ReadBody(String),
    #[error("FileReadError: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("JsonParseError: {0}")]
    JsonParse(#[from] serde_json::Error),
}
