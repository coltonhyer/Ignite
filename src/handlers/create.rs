use crate::error::AppError;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::info;
use uuid::Uuid;

const MAX_PAYLOAD_BYTES: usize = 10 * 1024; // 10KB
const MIN_TTL_SECONDS: i64 = 300; // 5 minutes
const MAX_TTL_SECONDS: i64 = 86400; // 24 hours
const DEFAULT_TTL_SECONDS: i64 = 3600; // 1 hour

#[derive(Deserialize)]
pub struct CreateSecretRequest {
    pub ciphertext: String,
    pub nonce: String,
    pub ttl_seconds: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateSecretResponse {
    pub id: String,
    pub expires_at: String,
}

pub async fn create_secret(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateSecretRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Base64 decode ciphertext and nonce
    let ciphertext = STANDARD.decode(&payload.ciphertext).map_err(|_| {
        AppError::InvalidRequest("Invalid base64 encoding for ciphertext".to_string())
    })?;

    let nonce = STANDARD
        .decode(&payload.nonce)
        .map_err(|_| AppError::InvalidRequest("Invalid base64 encoding for nonce".to_string()))?;

    // 2. Validate decoded ciphertext size
    if ciphertext.len() > MAX_PAYLOAD_BYTES {
        return Err(AppError::PayloadTooLarge);
    }

    // 3. Validate TTL
    let ttl = payload.ttl_seconds.unwrap_or(DEFAULT_TTL_SECONDS);
    if !(MIN_TTL_SECONDS..=MAX_TTL_SECONDS).contains(&ttl) {
        return Err(AppError::InvalidRequest(format!(
            "ttl_seconds must be between {} and {}",
            MIN_TTL_SECONDS, MAX_TTL_SECONDS
        )));
    }

    // 4. Generate ID and expiration
    let id = Uuid::new_v4().to_string();
    let expires_at = (Utc::now() + Duration::seconds(ttl)).to_rfc3339();

    // 5. Insert into database
    sqlx::query!(
        r#"
        INSERT INTO secrets (id, ciphertext, nonce, expires_at)
        VALUES (?, ?, ?, ?)
        "#,
        id,
        ciphertext,
        nonce,
        expires_at
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    // 6. Log database schema write operation at INFO level (ignoring sensitive info)
    info!(
        action = "create_secret",
        id = %id,
        expires_at = %expires_at,
        ciphertext_size = ciphertext.len(),
        "Secret created successfully"
    );

    // 7. Return 201 Created with JSON
    let response = CreateSecretResponse { id, expires_at };
    Ok((StatusCode::CREATED, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::{body::to_bytes, routing::post, Router};
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde_json::json;
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt; // for `oneshot` and `ready`

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory database");

        sqlx::query(
            "CREATE TABLE secrets (
                id TEXT PRIMARY KEY NOT NULL,
                ciphertext BLOB NOT NULL,
                nonce BLOB NOT NULL,
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create secrets table");

        pool
    }

    fn app(pool: SqlitePool) -> Router {
        Router::new()
            .route("/api/secrets", post(create_secret))
            .with_state(pool)
    }

    #[tokio::test]
    async fn test_create_secret_success() {
        let pool = setup_db().await;
        let router = app(pool);

        let ciphertext = STANDARD.encode(b"secret data");
        let nonce = STANDARD.encode(b"nonce");
        let payload = json!({
            "ciphertext": ciphertext,
            "nonce": nonce,
            "ttl_seconds": 3600
        });

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/secrets")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json_body: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json_body.get("id").is_some());
        assert!(json_body.get("expires_at").is_some());
    }

    #[tokio::test]
    async fn test_create_secret_missing_ttl() {
        let pool = setup_db().await;
        let router = app(pool);

        let ciphertext = STANDARD.encode(b"secret data");
        let nonce = STANDARD.encode(b"nonce");
        let payload = json!({
            "ciphertext": ciphertext,
            "nonce": nonce,
            "ttl_seconds": null
        }); // Testing the null TTL behavior specifically requested

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/secrets")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json_body: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json_body.get("id").is_some());
        assert!(json_body.get("expires_at").is_some());
    }

    #[tokio::test]
    async fn test_create_secret_payload_too_large() {
        let pool = setup_db().await;
        let router = app(pool);

        let large_data = vec![0u8; MAX_PAYLOAD_BYTES + 1];
        let ciphertext = STANDARD.encode(&large_data);
        let nonce = STANDARD.encode(b"nonce");
        let payload = json!({
            "ciphertext": ciphertext,
            "nonce": nonce
        });

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/secrets")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_create_secret_invalid_ttl() {
        let pool = setup_db().await;
        let router = app(pool);

        let ciphertext = STANDARD.encode(b"secret data");
        let nonce = STANDARD.encode(b"nonce");
        let payload = json!({
            "ciphertext": ciphertext,
            "nonce": nonce,
            "ttl_seconds": 10 // Invalid TTL
        });

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/secrets")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_secret_invalid_base64() {
        let pool = setup_db().await;
        let router = app(pool);

        let payload = json!({
            "ciphertext": "invalid_base64!",
            "nonce": STANDARD.encode(b"nonce")
        });

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/secrets")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_secret_missing_fields() {
        let pool = setup_db().await;
        let router = app(pool);

        // Missing nonce
        let payload = json!({
            "ciphertext": STANDARD.encode(b"secret")
        });

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/secrets")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY); // axum returns 422 for missing fields
    }
}
