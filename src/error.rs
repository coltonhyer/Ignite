use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    ServiceUnavailable,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::ServiceUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "error", "db": "disconnected" })),
            )
                .into_response(),
        }
    }
}
