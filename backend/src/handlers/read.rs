use crate::error::AppError;
use crate::store::SecretStore;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use tracing::info;
use uuid::Uuid;

use shared::ReadSecretResponse;

pub async fn read_secret(
    State(store): State<SecretStore>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate UUID format
    if Uuid::parse_str(&id).is_err() {
        return Err(AppError::InvalidRequest("Invalid UUID format".to_string()));
    }

    // 2. Execute atomic destructive read
    // The query returns `ciphertext` and `nonce` which are stored as BLOBs (Vec<u8> in Rust)
    let result = store
        .burn_secret(&id)
        .await
        .map_err(|e: sqlx::Error| AppError::Internal(anyhow::anyhow!(e)))?;

    // 3. Check if a row was returned
    match result {
        Some(row) => {
            // 4. Base64 encode the returned BLOBs
            let ciphertext_b64 = URL_SAFE_NO_PAD.encode(&row.ciphertext);
            let nonce_b64 = URL_SAFE_NO_PAD.encode(&row.nonce);

            // 5. Log the read operation at INFO level
            info!(
                action = "read_secret",
                id = %id,
                "Secret read and destroyed successfully"
            );

            // 6. Return 200 OK with JSON
            let response = ReadSecretResponse {
                ciphertext: ciphertext_b64,
                nonce: nonce_b64,
            };
            Ok((StatusCode::OK, Json(response)))
        }
        None => {
            // 7. If no row was deleted/returned, the secret doesn't exist or is expired
            Err(AppError::SecretNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use axum::{body::to_bytes, routing::delete, Router};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;
    use tower::ServiceExt;

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory database");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS secrets (
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
            .route("/api/secrets/{id}", delete(read_secret))
            .with_state(crate::store::SecretStore::new(pool))
    }

    #[tokio::test]
    async fn test_read_secret_success_and_second_read_not_found() {
        let pool = setup_db().await;
        let router = app(pool.clone());

        let id = Uuid::new_v4().to_string();
        let raw_ciphertext = b"secret data";
        let raw_nonce = b"nonce";
        let ciphertext_slice = raw_ciphertext.as_slice();
        let nonce_slice = raw_nonce.as_slice();

        // Insert a secret that expires in 1 hour
        sqlx::query(
            r#"
            INSERT INTO secrets (id, ciphertext, nonce, expires_at)
            VALUES (?, ?, ?, datetime('now', '+1 hour'))
            "#,
        )
        .bind(&id)
        .bind(ciphertext_slice)
        .bind(nonce_slice)
        .execute(&pool)
        .await
        .unwrap();

        // First read - should succeed and return 200 OK
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/secrets/{}", id))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json_body: ReadSecretResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json_body.ciphertext, URL_SAFE_NO_PAD.encode(raw_ciphertext));
        assert_eq!(json_body.nonce, URL_SAFE_NO_PAD.encode(raw_nonce));

        // Second read - should return 404 NOT FOUND (SecretNotFound)
        let response_second = router
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/secrets/{}", id))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response_second.status(), StatusCode::GONE);
    }

    #[tokio::test]
    async fn test_read_secret_invalid_uuid() {
        let pool = setup_db().await;
        let router = app(pool);

        let response = router
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/secrets/invalid-uuid-format")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // InvalidRequest maps to BAD_REQUEST (400)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_read_secret_expired() {
        let pool = setup_db().await;
        let router = app(pool.clone());

        let id = Uuid::new_v4().to_string();

        // Insert a secret that expired 1 hour ago
        let secret_slice = b"secret".as_slice();
        let nonce_slice = b"nonce".as_slice();
        sqlx::query(
            r#"
            INSERT INTO secrets (id, ciphertext, nonce, expires_at)
            VALUES (?, ?, ?, datetime('now', '-1 hour'))
            "#,
        )
        .bind(&id)
        .bind(secret_slice)
        .bind(nonce_slice)
        .execute(&pool)
        .await
        .unwrap();

        let response = router
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/secrets/{}", id))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::GONE);
    }
}
