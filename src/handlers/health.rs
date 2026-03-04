use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use sqlx::SqlitePool;

use crate::error::AppError;

pub async fn health_check(State(pool): State<SqlitePool>) -> Result<impl IntoResponse, AppError> {
    // Attempt to execute a simple query to verify database connectivity
    // Using `query` instead of `query!` to avoid sqlx checking at compile time without DB
    match sqlx::query("SELECT 1 as is_alive").fetch_one(&pool).await {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(json!({ "status": "ok", "db": "connected" })),
        )),
        Err(_) => Err(AppError::ServiceUnavailable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get, Router};
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt; // for `oneshot`

    async fn create_test_router(pool: SqlitePool) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .with_state(pool)
    }

    #[tokio::test]
    async fn test_health_check_ok() {
        // Create an in-memory database pool
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory database");

        let app = create_test_router(pool).await;

        let request = Request::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Check the Content-Type header
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        use axum::body::to_bytes;
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(body_json, json!({ "status": "ok", "db": "connected" }));
    }

    #[tokio::test]
    async fn test_health_check_error() {
        // Create an in-memory database pool, but close it to force a connection error
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory database");

        pool.close().await;

        let app = create_test_router(pool).await;

        let request = Request::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        // Check the Content-Type header
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        use axum::body::to_bytes;
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            body_json,
            json!({ "status": "error", "db": "disconnected" })
        );
    }
}
