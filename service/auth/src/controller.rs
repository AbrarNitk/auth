async fn from_body<T: serde::de::DeserializeOwned>(
    b: hyper::Body,
) -> Result<T, crate::error::AuthError> {
    let b = hyper::body::to_bytes(b)
        .await
        .map_err(|e| crate::error::AuthError::ReadBody(format!("{e}")))?;
    Ok(serde_json::from_slice(b.as_ref())?)
}

fn success(
    data: impl serde::Serialize,
) -> Result<hyper::Response<hyper::Body>, crate::error::AuthError> {
    #[derive(serde::Serialize)]
    struct ApiSuccess<T: serde::Serialize> {
        data: T,
        success: bool,
    }

    let resp = serde_json::to_vec(&ApiSuccess {
        data,
        success: true,
    })?;

    let mut response = hyper::Response::new(hyper::Body::from(resp));
    *response.status_mut() = hyper::StatusCode::OK;
    Ok(response)
}

fn error(
    message: String,
    status: hyper::StatusCode,
) -> Result<hyper::Response<hyper::Body>, crate::error::AuthError> {
    #[derive(serde::Serialize)]
    struct ApiError {
        message: String,
        success: bool,
    }

    let resp = serde_json::to_vec(&ApiError {
        success: false,
        message,
    })?;
    let mut response = hyper::Response::new(hyper::Body::from(resp));
    *response.status_mut() = status;
    Ok(response)
}

pub async fn api_handler(
    req: hyper::Request<hyper::Body>,
    db_pool: db::pg::DbPool,
) -> Result<hyper::Response<hyper::Body>, crate::error::AuthError> {
    let (p, b) = req.into_parts();
    let _start = std::time::Instant::now();
    match (&p.method, p.uri.path()) {
        (&hyper::Method::POST, "/api/auth/send-otp/") => {
            match crate::otp::send_otp(from_body(b).await?, db_pool).await {
                Ok(response) => success(response),
                Err(err) => {
                    println!("err:send_otp: {}", err);
                    error(
                        "server error".to_string(),
                        hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                }
            }
        }
        (&hyper::Method::POST, "/api/auth/resend-otp/") => {
            match crate::otp::resend_otp(from_body(b).await?, db_pool).await {
                Ok(response) => success(response),
                Err(err) => {
                    println!("err:re_send_otp: {}", err);
                    error(
                        "server error".to_string(),
                        hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                }
            }
        }
        (&hyper::Method::POST, "/api/auth/verify-otp/") => {
            match crate::otp::verify_otp(from_body(b).await?, db_pool).await {
                Ok(response) => success(response),
                Err(err) => {
                    println!("err:re_send_otp: {}", err);
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
    req: hyper::Request<hyper::Body>,
    db_pool: db::pg::DbPool,
) -> Result<hyper::Response<hyper::Body>, crate::error::AuthError> {

    // Note: API handler
    if req.uri().path().starts_with("/api/auth/") {
        return api_handler(req, db_pool).await
    }

    // OAuth handler
    match req.uri().path() {
        "/auth/github/login/" => Ok(crate::github::login(&req).await.unwrap()),
        "/auth/github/callback/" => Ok(crate::github::callback(&req).await.unwrap()),
        // Some(p) if p.eq("gitlab") => Ok(hyper::Response::new(hyper::Body::empty())),
        // Some(p) if p.eq("google") => Ok(hyper::Response::new(hyper::Body::empty())),
        // Some(p) if p.eq("twitter") => Ok(hyper::Response::new(hyper::Body::empty())),
        // Some(p) if p.eq("discord") => Ok(hyper::Response::new(hyper::Body::empty())),
        // Some(p) if p.eq("linkedin") => Ok(hyper::Response::new(hyper::Body::empty())),
        _ => {
            let bytes = tokio::fs::read("service/auth/login.html").await?;
            Ok(hyper::Response::new(hyper::Body::from(bytes)))
        }
    }
}

pub fn response(body: String, status: hyper::StatusCode) -> hyper::Response<hyper::Body> {
    let mut response = hyper::Response::new(hyper::Body::from(body));
    *response.status_mut() = status;
    response.headers_mut().append(
        hyper::header::CONTENT_TYPE,
        hyper::http::HeaderValue::from_static("application/json"),
    );
    response
}
