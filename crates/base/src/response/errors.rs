use axum::{http::StatusCode, response::IntoResponse};
use std::fmt::{Debug, Display};

#[derive(serde::Serialize)]
pub enum ErrorCode {
    NotFound,
    InternalServerError,
    BadRequest,
    UnAuthorized,
    Forbidden,
    Conflict,
}

impl From<ErrorCode> for StatusCode {
    fn from(value: ErrorCode) -> Self {
        match value {
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::BadRequest => StatusCode::BAD_REQUEST,
            ErrorCode::UnAuthorized => StatusCode::UNAUTHORIZED,
            ErrorCode::Forbidden => StatusCode::FORBIDDEN,
            ErrorCode::Conflict => StatusCode::CONFLICT,
        }
    }
}

#[derive(serde::Serialize)]
pub struct ErrorResponseData {
    trace_id: String,
    // Note: it can be epoch, but it does not make sense because people need to read this
    timestamp: chrono::DateTime<chrono::Utc>,
    code: ErrorCode,
    #[serde(skip)]
    status: StatusCode,

    // User friendly messages
    message: String,

    // Specific description in the context of the error
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    // technical details of the error with the backtrace
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    success: bool,
    error: ErrorResponseData,
}

impl IntoResponse for ErrorResponseData {
    fn into_response(self) -> axum::response::Response {
        let status = self.status;
        let error_response = ErrorResponse {
            success: false,
            error: self,
        };
        (
            status,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            axum::Json(error_response),
        )
            .into_response()
    }
}

pub trait ResponseError: Debug + Display + std::error::Error {
    fn error_code(&self) -> ErrorCode;
    fn status_code(&self) -> StatusCode {
        self.error_code().into()
    }
    fn message(&self) -> String {
        self.to_string() // Default to thiserror message
    }

    fn desc(&self) -> Option<String> {
        Some(self.to_string()) // Default to thiserror message
    }

    fn details(&self) -> Option<String> {
        let mut backtrace = vec![];
        let mut error: &dyn std::error::Error = &self;
        while let Some(source) = error.source() {
            backtrace.push(source);
            error = source;
        }

        if backtrace.is_empty() {
            return None;
        }

        Some(
            backtrace
                .into_iter()
                .map(|err| err.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

pub fn error<Err>(trace_id: &str, err: Err) -> axum::response::Response
where
    Err: ResponseError,
{
    ErrorResponseData {
        trace_id: trace_id.to_string(),
        timestamp: chrono::Utc::now(),
        code: err.error_code(),
        status: err.status_code(),
        message: err.message(),
        description: err.desc(),
        details: err.details(),
    }
    .into_response()
}
