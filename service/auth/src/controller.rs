async fn from_body<T: serde::de::DeserializeOwned>(
    b: hyper::Body,
) -> Result<T, crate::error::AuthError> {
    let b = hyper::body::to_bytes(b)
        .await
        .map_err(|e| crate::error::AuthError::ReadBody(format!("{e}")))?;
    Ok(serde_json::from_slice(b.as_ref())?)
}

pub async fn routes(
    req: hyper::Request<hyper::Body>,
    db_pool: db::pg::DbPool,
) -> Result<hyper::Response<hyper::Body>, crate::error::AuthError> {
    let (p, b) = req.into_parts();
    let start = std::time::Instant::now();
    match (&p.method, p.uri.path()) {
        (&hyper::Method::POST, "/api/auth/send-otp/") => {
            match crate::otp::send_otp(from_body(b).await?, db_pool).await {
                Ok(_) => {
                    println!("Hello Send OTP");
                }
                Err(e) => println!("Error in sending email: {}", e),
            };
            Ok(hyper::Response::new(hyper::Body::from("")))
        }
        (&hyper::Method::POST, "/api/auth/resend-otp/") => {
            match crate::otp::resend_otp(from_body(b).await?, db_pool).await {
                Ok(_) => {
                    println!("Hello resend OTP");
                }
                Err(e) => println!("Error in sending email: {}", e),
            };
            println!("Hello resend OTP");
            Ok(hyper::Response::new(hyper::Body::from("")))
        }
        (&hyper::Method::POST, "/api/auth/verify-otp/") => {
            match crate::otp::verify_otp(from_body(b).await?, db_pool).await {
                Ok(_) => {
                    println!("Hello verify OTP");
                }
                Err(e) => println!("Error in sending email: {}", e),
            };
            Ok(hyper::Response::new(hyper::Body::from("")))
        }
        _ => Ok(hyper::Response::new(hyper::Body::from(""))),
    }

    // let path = req.uri().path();
    // let query = url::form_urlencoded::parse(req.uri().query().unwrap_or("").as_bytes())
    //     .into_owned()
    //     .collect::<std::collections::HashMap<String, String>>();
    // match query.get("platform") {
    //     Some(p) if p.eq("github") => Ok(crate::github::login(&req).await.unwrap()),
    //     Some(p) if p.eq("gitlab") => Ok(hyper::Response::new(hyper::Body::empty())),
    //     Some(p) if p.eq("google") => Ok(hyper::Response::new(hyper::Body::empty())),
    //     Some(p) if p.eq("twitter") => Ok(hyper::Response::new(hyper::Body::empty())),
    //     Some(p) if p.eq("discord") => Ok(hyper::Response::new(hyper::Body::empty())),
    //     Some(p) if p.eq("linkedin") => Ok(hyper::Response::new(hyper::Body::empty())),
    //     _ => {
    //         let bytes = tokio::fs::read("service/auth/login.html").await?;
    //         Ok(hyper::Response::new(hyper::Body::from(bytes)))
    //     }
    // }

    //Ok(hyper::Response::new(hyper::Body::from("")))
}
