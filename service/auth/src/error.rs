#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("FileReadError: {0}")]
    FileReadError(#[from] std::io::Error),
}
