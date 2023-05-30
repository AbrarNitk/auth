pub async fn routes(
    req: hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, crate::error::AuthError> {
    // let path = req.uri().path();
    let query = url::form_urlencoded::parse(req.uri().query().unwrap_or("").as_bytes())
        .into_owned()
        .collect::<std::collections::HashMap<String, String>>();
    if !query.contains_key("platform") {
        let bytes = tokio::fs::read("service/auth/login.html").await?;
        Ok(hyper::Response::new(hyper::Body::from(bytes)))
    } else {
        Ok(hyper::Response::new(hyper::Body::empty()))
    }
}
