pub mod communication;
pub mod controller;
pub mod error;
mod github;
pub mod jwt;
pub mod otp;
pub mod utils;

pub(crate) static BREVO_API_KEY: once_cell::sync::Lazy<String> = {
    once_cell::sync::Lazy::new(|| match std::env::var("BREVO_API_KEY") {
        Ok(val) => val,
        Err(e) => panic!("{}{}", "BREVO_API_KEY not found in env ", e),
    })
};

#[macro_export]
macro_rules! not_found {
    () => {
        crate::controller::response("Not Found".to_owned(), hyper::StatusCode::NOT_FOUND)
    };

    ($body:expr) => {
        crate::controller::response($body, hyper::StatusCode::NOT_FOUND)
    };
}

#[macro_export]
macro_rules! server_error {
    () => {
        crate::controller::response(
            "Server Error".to_owned(),
            hyper::StatusCode::INTERNAL_SERVER_ERROR,
        )
    };
}

#[macro_export]
macro_rules! ok {
    ($body:expr) => {
        crate::controller::response($body, hyper::StatusCode::OK)
    };
}
