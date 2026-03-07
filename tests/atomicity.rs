use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use ignite::{
    handlers::{create::CreateSecretResponse, read::ReadSecretResponse},
    router::create_router,
};
use serde_json::json;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tokio::time::{sleep, Duration};
use tower::ServiceExt;
use uuid::Uuid;

// Helper to setup a fresh database and router for each test
async fn setup_app() -> (axum::Router, SqlitePool) {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory database");

    // Run migrations on the fresh in-memory database
    ignite::migrate::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    let router = create_router(pool.clone());
    (router, pool)
}

#[tokio::test]
async fn test_case_1_happy_path() {
    let (app, _) = setup_app().await;

    // Create a secret
    let ciphertext = STANDARD.encode(b"secret data");
    let nonce = STANDARD.encode(b"nonce");
    let payload = json!({
        "ciphertext": ciphertext,
        "nonce": nonce,
        "ttl_seconds": 3600
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/secrets")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    // clone app for multiple requests
    let create_res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let body = to_bytes(create_res.into_body(), usize::MAX).await.unwrap();
    let create_json: CreateSecretResponse = serde_json::from_slice(&body).unwrap();
    let secret_id = create_json.id;

    // Read it once
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/secrets/{}", secret_id))
        .body(Body::empty())
        .unwrap();

    let read_res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(read_res.status(), StatusCode::OK);

    let body = to_bytes(read_res.into_body(), usize::MAX).await.unwrap();
    let read_json: ReadSecretResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(read_json.ciphertext, ciphertext);
    assert_eq!(read_json.nonce, nonce);

    // Read again - assert 404 Not Found
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/secrets/{}", secret_id))
        .body(Body::empty())
        .unwrap();

    let read_again_res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(read_again_res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_case_2_never_existed() {
    let (app, _) = setup_app().await;
    let fake_id = Uuid::new_v4().to_string();

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/secrets/{}", fake_id))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_case_3_invalid_format() {
    let (app, _) = setup_app().await;

    let req = Request::builder()
        .method("DELETE")
        .uri("/api/secrets/garbage-string")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_case_4_expired_secret() {
    let (app, pool) = setup_app().await;

    let id = Uuid::new_v4().to_string();
    let ciphertext = b"secret".to_vec();
    let nonce = b"nonce".to_vec();

    // Insert a secret expiring in 1 second using direct db interface, not query macro to avoid needing DATABASE_URL
    sqlx::query(
        r#"
        INSERT INTO secrets (id, ciphertext, nonce, expires_at)
        VALUES (?1, ?2, ?3, datetime('now', '+1 seconds'))
        "#,
    )
    .bind(&id)
    .bind(ciphertext)
    .bind(nonce)
    .execute(&pool)
    .await
    .unwrap();

    // Sleep for slightly more than 1 second to ensure expiry
    sleep(Duration::from_millis(1100)).await;

    // Read it
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/secrets/{}", id))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    // Because it's expired, DELETE ... WHERE expires_at > datetime('now') returns 0 rows
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_case_5_payload_validation() {
    let (app, _) = setup_app().await;

    let large_data = vec![0u8; 10 * 1024 + 1]; // > 10KB
    let ciphertext = STANDARD.encode(&large_data);
    let nonce = STANDARD.encode(b"nonce");
    let payload = json!({
        "ciphertext": ciphertext,
        "nonce": nonce,
        "ttl_seconds": 3600
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/secrets")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn test_case_6_ttl_validation() {
    let (app, _) = setup_app().await;

    let ciphertext = STANDARD.encode(b"secret data");
    let nonce = STANDARD.encode(b"nonce");
    let payload = json!({
        "ciphertext": ciphertext,
        "nonce": nonce,
        "ttl_seconds": 60 // Below 300 minimum
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/secrets")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
