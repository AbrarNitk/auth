pub async fn handler(
    req: hyper::Request<hyper::Body>,
    db_pool: db::pg::DbPool,
) -> Result<hyper::Response<hyper::Body>, http_service::errors::RouteError> {
    tracing::info!(method = req.method().as_str(), path = req.uri().path());

    if req.uri().path().starts_with("/auth/health") {
        let mut response = hyper::Response::new(hyper::Body::empty());
        *response.body_mut() = hyper::Body::from(serde_json::json!({"success": true}).to_string());
        *response.status_mut() = hyper::StatusCode::OK;
        response.headers_mut().append(
            hyper::header::CONTENT_TYPE,
            hyper::http::HeaderValue::from_str("application/json").unwrap(), // TODO: Remove unwrap
        );
        return Ok(response);
    }

    if req.uri().path().starts_with("/auth/") || req.uri().path().starts_with("/v1/api/auth/") {
        // todo: handle the error here only
        return Ok(auth::controller::routes(req, db_pool).await?);
    }

    if req.uri().path().starts_with("/api/ai/") {
        return Ok(ai::apis::handle(req, db_pool).await?);
    }

    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/") => Ok(hyper::Response::new(hyper::Body::from(
            tokio::fs::read("service/index.html").await?,
        ))),
        (&hyper::Method::POST, "/") => {
            let mut response = hyper::Response::new(hyper::Body::empty());
            *response.body_mut() = hyper::Body::from("POST Response");
            *response.status_mut() = hyper::StatusCode::OK;
            response.headers_mut().append(
                hyper::header::CONTENT_TYPE,
                hyper::http::HeaderValue::from_str("application/json").unwrap(), // TODO: Remove unwrap
            );
            Ok(response)
        }

        _ => Ok(auth::not_found!(serde_json::json!(
                {"message": format!("route not found: {}",req.uri().path()),"success": false})
        .to_string())),
    }
}
