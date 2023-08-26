#[derive(serde::Deserialize)]
pub struct SendOtpReq {
    pub email: Option<String>,
    pub phone: Option<String>,
}

pub struct VerifyOtpReq {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub otp: u32,
}

pub async fn routes(
    req: hyper::Request<hyper::Body>,
    db_pool: db::pg::DbPool,
) -> Result<hyper::Response<hyper::Body>, crate::error::AuthError> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::POST, "/api/auth/send-otp/") => {
            match crate::otp::send_otp("manishmsiclub@gmail.com", "Manish Jain", db_pool).await {
                Ok(_) => {
                    println!("Hello Send OTP");
                }
                Err(e) => println!("Error in sending email: {}", e),
            };
            Ok(hyper::Response::new(hyper::Body::from("")))
        }
        (&hyper::Method::POST, "/api/auth/resend-otp/") => {
            match crate::otp::resend_otp("manishmsiclub@gmail.com", db_pool).await {
                Ok(_) => {
                    println!("Hello resend OTP");
                }
                Err(e) => println!("Error in sending email: {}", e),
            };
            println!("Hello resend OTP");
            Ok(hyper::Response::new(hyper::Body::from("")))
        }
        (&hyper::Method::POST, "/api/auth/verify-otp/") => {
            println!("Hello verify OTP");
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
