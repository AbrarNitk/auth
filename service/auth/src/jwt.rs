// const BEARER: &str = "Bearer ";
const JWT_EXPIRY: u64 = 30 * 24 * 60 * 60; // 30 days
const JWT_SECRET: &[u8] = b"secret";

#[derive(thiserror::Error, Debug)]
pub enum JWTError {
    #[error("JWTDecodeError: {0}")]
    JWTEncodeError(#[from] jsonwebtoken::errors::Error),
    #[error("TokenHeader")]
    TokenHeaderFormat,
    #[error("TokenHeaderNotFound")]
    TokenHeaderNotFound,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    sub: String,
    iat: usize,
    exp: usize,
}

pub fn create_jwt(uid: String) -> Result<String, JWTError> {
    let now = jsonwebtoken::get_current_timestamp();
    let claims = Claims {
        sub: uid,
        iat: now as usize,
        exp: (now + JWT_EXPIRY) as usize,
    };
    let jwt = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(JWT_SECRET),
    )?;
    Ok(jwt)
}

pub fn decode_jwt(
    headers: &hyper::HeaderMap<hyper::header::HeaderValue>,
) -> Result<String, JWTError> {
    let header = match headers.get(hyper::header::AUTHORIZATION) {
        Some(h) => h,
        None => return Err(JWTError::TokenHeaderNotFound),
    };
    let token = header.to_str().map_err(|_| JWTError::TokenHeaderFormat)?; // Note: all the chars in the header token should be ascii
    let decode_claims = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(JWT_SECRET),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512),
    )?;
    Ok(decode_claims.claims.sub)
}
