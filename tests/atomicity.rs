use std::net::SocketAddr;

use axum::{
    body::{to_bytes, Body},
    extract::ConnectInfo,
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

const TEST_ADDR: ConnectInfo<SocketAddr> = ConnectInfo(SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
    0,
));

/// Build a POST /api/secrets request with the given IP (via x-forwarded-for).
fn create_secret_request(ip: &str, ciphertext: &str, nonce: &str) -> Request<Body> {
    let payload = json!({
        "ciphertext": ciphertext,
        "nonce": nonce,
        "ttl_seconds": 3600
    });
    Request::builder()
        .method("POST")
        .uri("/api/secrets")
        .header("content-type", "application/json")
        .header("x-forwarded-for", ip)
        .body(Body::from(payload.to_string()))
        .unwrap()
}

/// Build a DELETE /api/secrets/{id} request with the given IP (via x-forwarded-for).
fn burn_secret_request(id: &str, ip: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(format!("/api/secrets/{}", id))
        .header("x-forwarded-for", ip)
        .body(Body::empty())
        .unwrap()
}

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
        .extension(TEST_ADDR)
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
        .extension(TEST_ADDR)
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
        .extension(TEST_ADDR)
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
        .extension(TEST_ADDR)
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
        .extension(TEST_ADDR)
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
        .extension(TEST_ADDR)
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
        .extension(TEST_ADDR)
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
        .extension(TEST_ADDR)
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_case_7_rate_limit_post() {
    let (app, _) = setup_app().await;

    let ciphertext = STANDARD.encode(b"secret data");
    let nonce = STANDARD.encode(b"nonce");

    // Send 10 requests (burst size) — all should succeed
    for _ in 0..10 {
        let res = app
            .clone()
            .oneshot(create_secret_request("10.0.0.1", &ciphertext, &nonce))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    // 11th request should be rate limited
    let res = app
        .clone()
        .oneshot(create_secret_request("10.0.0.1", &ciphertext, &nonce))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

    // Verify Retry-After header is present and numeric
    let retry_after = res
        .headers()
        .get("retry-after")
        .expect("Retry-After header must be present");
    let retry_secs: u64 = retry_after
        .to_str()
        .unwrap()
        .parse()
        .expect("Retry-After must be numeric");
    assert!(retry_secs > 0);

    // Verify JSON error body
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_body["code"], "RATE_LIMITED");
}

#[tokio::test]
async fn test_case_8_rate_limit_independent_ips() {
    let (app, _) = setup_app().await;

    let ciphertext = STANDARD.encode(b"secret data");
    let nonce = STANDARD.encode(b"nonce");

    // Exhaust rate limit for IP 10.0.0.2
    for _ in 0..10 {
        app.clone()
            .oneshot(create_secret_request("10.0.0.2", &ciphertext, &nonce))
            .await
            .unwrap();
    }

    // A different IP (10.0.0.3) should still be allowed
    let res = app
        .clone()
        .oneshot(create_secret_request("10.0.0.3", &ciphertext, &nonce))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_case_9_rate_limit_delete() {
    let (app, pool) = setup_app().await;

    // Pre-create 31 secrets directly in the DB
    let mut ids = Vec::new();
    for _ in 0..31 {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO secrets (id, ciphertext, nonce, expires_at)
            VALUES (?1, ?2, ?3, datetime('now', '+3600 seconds'))
            "#,
        )
        .bind(&id)
        .bind(b"cipher".to_vec())
        .bind(b"nonce".to_vec())
        .execute(&pool)
        .await
        .unwrap();
        ids.push(id);
    }

    // Send 30 DELETE requests (burst size) — all should succeed
    for id in &ids[..30] {
        let res = app
            .clone()
            .oneshot(burn_secret_request(id, "10.0.0.4"))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    // 31st DELETE should be rate limited
    let res = app
        .clone()
        .oneshot(burn_secret_request(&ids[30], "10.0.0.4"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

    let retry_after = res
        .headers()
        .get("retry-after")
        .expect("Retry-After header must be present");
    let retry_secs: u64 = retry_after
        .to_str()
        .unwrap()
        .parse()
        .expect("Retry-After must be numeric");
    assert!(retry_secs > 0);
}
