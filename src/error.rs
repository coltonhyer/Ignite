use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

fn sanitize_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    PayloadTooLarge,
    InvalidRequest(String),
    SecretNotFound,
    NotFound,
    Internal(anyhow::Error),
    ServiceUnavailable,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, code) = match self {
            AppError::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "Payload exceeds 10KB limit".to_string(),
                "PAYLOAD_TOO_LARGE",
            ),
            AppError::InvalidRequest(msg) => (
                StatusCode::BAD_REQUEST,
                sanitize_html(&msg),
                "INVALID_REQUEST",
            ),
            AppError::SecretNotFound => (
                StatusCode::NOT_FOUND,
                "Secret does not exist or has already been destroyed".to_string(),
                "SECRET_NOT_FOUND",
            ),
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "Resource not found".to_string(),
                "NOT_FOUND",
            ),
            AppError::Internal(err) => {
                tracing::error!("Internal server error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                    "INTERNAL",
                )
            }
            // Keep the old response for ServiceUnavailable as required by health tests
            AppError::ServiceUnavailable => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({ "status": "error", "db": "disconnected" })),
                )
                    .into_response();
            }
        };

        (
            status,
            Json(json!({ "error": error_message, "code": code })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use serde_json::Value;

    async fn get_response_parts(err: AppError) -> (StatusCode, Value) {
        let response = err.into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();
        (status, json_body)
    }

    #[tokio::test]
    async fn test_payload_too_large() {
        let err = AppError::PayloadTooLarge;
        let (status, body) = get_response_parts(err).await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(
            body,
            json!({ "error": "Payload exceeds 10KB limit", "code": "PAYLOAD_TOO_LARGE" })
        );
    }

    #[tokio::test]
    async fn test_invalid_request() {
        let err = AppError::InvalidRequest("Missing field".to_string());
        let (status, body) = get_response_parts(err).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            body,
            json!({ "error": "Missing field", "code": "INVALID_REQUEST" })
        );
    }

    #[tokio::test]
    async fn test_invalid_request_sanitization() {
        let err = AppError::InvalidRequest("<script>alert('1')</script> & \"more\"".to_string());
        let (status, body) = get_response_parts(err).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            body,
            json!({ "error": "&lt;script&gt;alert(&#x27;1&#x27;)&lt;/script&gt; &amp; &quot;more&quot;", "code": "INVALID_REQUEST" })
        );
    }

    #[tokio::test]
    async fn test_secret_not_found() {
        let err = AppError::SecretNotFound;
        let (status, body) = get_response_parts(err).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(
            body,
            json!({ "error": "Secret does not exist or has already been destroyed", "code": "SECRET_NOT_FOUND" })
        );
    }

    #[tokio::test]
    async fn test_not_found() {
        let err = AppError::NotFound;
        let (status, body) = get_response_parts(err).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(
            body,
            json!({ "error": "Resource not found", "code": "NOT_FOUND" })
        );
    }

    #[tokio::test]
    async fn test_internal() {
        let err = AppError::Internal(anyhow::anyhow!("Something went wrong"));
        let (status, body) = get_response_parts(err).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            body,
            json!({ "error": "Internal server error", "code": "INTERNAL" })
        );
    }

    #[tokio::test]
    async fn test_service_unavailable() {
        let err = AppError::ServiceUnavailable;
        let (status, body) = get_response_parts(err).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body, json!({ "status": "error", "db": "disconnected" }));
    }
}
