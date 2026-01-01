use http_body_util::BodyExt;
use hyper::body::Incoming;

async fn from_body<T: serde::de::DeserializeOwned>(
    req: Incoming,
) -> Result<T, crate::error::AuthError> {
    let collected_body = req
        .collect()
        .await
        .map_err(|e| crate::error::AuthError::ReadBody(format!("{e}")))?
        .to_bytes();

    Ok(serde_json::from_slice(&collected_body)?)
}

fn success(
    data: impl serde::Serialize,
) -> Result<hyper::Response<Vec<u8>>, crate::error::AuthError> {
    #[derive(serde::Serialize)]
    struct ApiSuccess<T: serde::Serialize> {
        data: T,
        success: bool,
    }

    let resp = serde_json::to_vec(&ApiSuccess {
        data,
        success: true,
    })?;

    let mut response = hyper::Response::new(resp);
    *response.status_mut() = hyper::StatusCode::OK;
    response.headers_mut().append(
        hyper::header::CONTENT_TYPE,
        hyper::http::HeaderValue::from_str("application/json").unwrap(), // TODO: Remove unwrap
    );
    Ok(response)
}

fn error(
    message: String,
    status: hyper::StatusCode,
) -> Result<hyper::Response<Vec<u8>>, crate::error::AuthError> {
    #[derive(serde::Serialize)]
    struct ApiError {
        message: String,
        success: bool,
    }

    let resp = serde_json::to_vec(&ApiError {
        success: false,
        message,
    })?;
    let mut response = hyper::Response::new(resp);
    *response.status_mut() = status;
    Ok(response)
}

pub async fn api_handler(
    req: hyper::Request<Incoming>,
    db_pool: db::pg::DbPool,
) -> Result<hyper::Response<Vec<u8>>, crate::error::AuthError> {
    let (p, b) = req.into_parts();
    let _start = std::time::Instant::now();
    match (&p.method, p.uri.path()) {
        (&hyper::Method::POST, "/v1/api/auth/send-otp/") => {
            match crate::otp::send_otp(from_body(b).await?, db_pool).await {
                Ok(response) => success(response),
                Err(err) => {
                    tracing::error!(mesage = "err:send_otp", error = err.to_string());
                    error(
                        "server error".to_string(),
                        hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                }
            }
        }
        (&hyper::Method::POST, "/v1/api/auth/resend-otp/") => {
            match crate::otp::resend_otp(from_body(b).await?, db_pool).await {
                Ok(response) => success(response),
                Err(err) => {
                    tracing::error!(message = "err:re_send_otp", error = err.to_string());
                    error(
                        "server error".to_string(),
                        hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                }
            }
        }
        (&hyper::Method::POST, "/v1/api/auth/verify-otp/") => {
            match crate::otp::verify_otp(from_body(b).await?, db_pool).await {
                Ok(response) => success(response),
                Err(err) => {
                    tracing::error!(message = "err:re_send_otp", error = err.to_string());
                    error(
                        "server error".to_string(),
                        hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                }
            }
        }
        _ => Ok(crate::not_found!(serde_json::json!(
                {"message": format!("route not found: {}", p.uri.path()),"success": false})
        .to_string())),
    }
}

// todo: remove unwrap and add logging
pub async fn routes(
    req: hyper::Request<Incoming>,
    db_pool: db::pg::DbPool,
) -> Result<hyper::Response<Vec<u8>>, crate::error::AuthError> {
    // Note: API handler
    if req.uri().path().starts_with("/v1/api/auth/") {
        return api_handler(req, db_pool).await;
    }

    // OAuth handler
    match req.uri().path() {
        "/auth/github/login/" => Ok(crate::github::login(&req).await.unwrap()),
        "/auth/github/callback/" => Ok(crate::github::callback(&req).await.unwrap()),

        // Note: send the cookies starts with auth-
        "/auth/get-identities/" => {
            let (_p, b) = req.into_parts();
            match crate::get_identities::get_identities(from_body(b).await?).await {
                Ok(response) => success(response),
                Err(err) => {
                    println!("err:re_send_otp: {:?}", err);
                    error(
                        "server error".to_string(),
                        hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                }
            }
        }
        // Some(p) if p.eq("gitlab") => Ok(hyper::Response::new(hyper::Body::empty())),
        // Some(p) if p.eq("google") => Ok(hyper::Response::new(hyper::Body::empty())),
        // Some(p) if p.eq("twitter") => Ok(hyper::Response::new(hyper::Body::empty())),
        // Some(p) if p.eq("discord") => Ok(hyper::Response::new(hyper::Body::empty())),
        // Some(p) if p.eq("linkedin") => Ok(hyper::Response::new(hyper::Body::empty())),
        _ => {
            let bytes = tokio::fs::read("service/auth/login.html").await?;
            Ok(hyper::Response::new(bytes))
        }
    }
}

pub fn response(body: String, status: hyper::StatusCode) -> hyper::Response<Vec<u8>> {
    let mut response = hyper::Response::new(body.into_bytes());
    *response.status_mut() = status;
    response.headers_mut().append(
        hyper::header::CONTENT_TYPE,
        hyper::http::HeaderValue::from_static("application/json"),
    );
    response
}
