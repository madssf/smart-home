use std::error::Error;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use log::error;

struct ErrorType {
    status_code: StatusCode,
    message: String,
}
#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
}

// The helper function creates a tuple containing the status code
// and the error, which implements `IntoResponse`.
pub fn error_response(error: String, status_code: StatusCode) -> impl IntoResponse {
    ErrorType {
        status_code,
        message: error,
    }
}
pub fn internal_server_error<E>(error: E) -> impl IntoResponse
where
    E: Error,
{
    error!("{:?}", error);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal server error: {:?}", error),
    )
}

// Implement `IntoResponse` for the tuple `(StatusCode, ErrorResponse)`
impl IntoResponse for ErrorType {
    fn into_response(self) -> Response {
        // Only the `ErrorResponse` part is serialized into JSON.
        let body = Json(self.message);
        // The status code is used here, but not included in the JSON body.
        (self.status_code, body).into_response()
    }
}
