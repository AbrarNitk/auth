pub mod communication;
pub mod controller;
pub mod error;
mod github;
pub mod otp;
pub mod utils;

pub(crate) static BREVO_API_KEY: once_cell::sync::Lazy<String> = {
    once_cell::sync::Lazy::new(|| match std::env::var("BREVO_API_KEY") {
        Ok(val) => val,
        Err(e) => panic!("{}{}", "BREVO_API_KEY not found in env ", e),
    })
};
