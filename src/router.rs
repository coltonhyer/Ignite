use axum::{routing::get, Router};
use sqlx::SqlitePool;
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    trace::TraceLayer,
};

/// Creates the Axum application router with standard middleware attached.
///
/// Middleware stack includes:
/// - Request ID (x-request-id generated via UUID v4)
/// - Tracing (request/response logging)
/// - CORS (permissive for local development)
pub fn create_router(pool: SqlitePool) -> Router {
    Router::new()
        // Temporary health check route to verify server is running
        .route("/health", get(|| async { "OK" }))
        // Middleware is applied from bottom to top
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http().on_body_chunk(()))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        // Wire the SqlitePool into Axum State
        .with_state(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_create_router() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory database");

        let _router = create_router(pool);
        // If we reached here without panicking, the router creation was successful
    }
}
