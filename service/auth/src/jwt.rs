#[derive(thiserror::Error, Debug)]
pub enum JWTError {}

pub fn create_jwt(claims: impl serde::Serialize) -> Result<String, JWTError> {
    Ok("".to_string())
}
