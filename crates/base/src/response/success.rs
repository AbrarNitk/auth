use axum::response::IntoResponse;
use std::str::FromStr;

pub fn success<T: serde::Serialize>(data: T) -> axum::response::Response {
    success_with_status(data, axum::http::StatusCode::OK)
}

pub fn success_with_status<T: serde::Serialize>(
    data: T,
    status_code: axum::http::StatusCode,
) -> axum::response::Response {
    #[derive(serde::Serialize)]
    struct ApiSuccess<T> {
        success: bool,
        data: T,
    }
    (
        status_code,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        axum::Json(ApiSuccess {
            success: true,
            data,
        }),
    )
        .into_response()
}

pub fn with_headers<T: serde::Serialize>(
    data: T,
    headers: &Vec<(String, String)>,
    status_code: axum::http::StatusCode,
) -> axum::response::Response {
    #[derive(serde::Serialize)]
    struct ApiSuccess<T> {
        success: bool,
        data: T,
    }

    // set the extra headers
    let mut h = axum::http::header::HeaderMap::new();
    for (name, value) in headers {
        h.insert(
            axum::http::HeaderName::from_str(name.as_str()).unwrap(),
            value.parse().unwrap(),
        );
    }

    (
        status_code,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        h,
        axum::Json(ApiSuccess {
            success: true,
            data,
        }),
    )
        .into_response()
}
