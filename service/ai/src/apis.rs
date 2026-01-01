use hyper::body::Incoming;
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AIError {
    #[error("JWTError: {}", _0)]
    JWT(#[from] auth::jwt::JWTError),
}

pub async fn handle(
    req: hyper::Request<Incoming>,
    _db_pool: db::pg::DbPool,
) -> Result<hyper::Response<Vec<u8>>, AIError> {
    let uid = auth::jwt::decode_jwt(req.headers())?;
    let mut response = hyper::Response::new(
        json!({
            "data": {
                "uid": uid,
            },
            "success": true
        })
        .to_string()
        .into_bytes(),
    );
    *response.status_mut() = hyper::StatusCode::OK;
    response.headers_mut().append(
        hyper::header::CONTENT_TYPE,
        hyper::http::HeaderValue::from_str("application/json").unwrap(), // TODO: Remove unwrap
    );
    Ok(response)
}
