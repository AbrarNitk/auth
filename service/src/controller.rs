#[derive(Default, Debug, serde::Serialize)]
pub struct GetProfileResponse {}

#[derive(thiserror::Error, Debug)]
pub enum GetProfileError {}

pub fn get_user_profile() -> Result<GetProfileResponse, GetProfileError> {
    Ok(GetProfileResponse::default())
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
