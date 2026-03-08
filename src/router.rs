use axum::{
    body::Body,
    http::Response,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use sqlx::SqlitePool;
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
    GovernorError, GovernorLayer,
};
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::error::AppError;

fn governor_error_handler(error: GovernorError) -> Response<Body> {
    match error {
        GovernorError::TooManyRequests { wait_time, .. } => {
            AppError::RateLimited { retry_after_secs: wait_time }.into_response()
        }
        _ => AppError::Internal(anyhow::anyhow!(error.to_string())).into_response(),
    }
}

/// Creates the Axum application router with standard middleware attached.
///
/// Middleware stack includes:
/// - Request ID (x-request-id generated via UUID v4)
/// - Tracing (request/response logging)
/// - CORS (permissive for local development)
/// - Per-route rate limiting via tower_governor
pub fn create_router(pool: SqlitePool) -> Router {
    // POST /api/secrets: 10 req/min (replenish 1 token every 6 seconds, burst 10)
    let create_governor = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(6)
        .burst_size(10)
        .use_headers()
        .finish()
        .unwrap();

    // DELETE /api/secrets/{id}: 30 req/min (replenish 1 token every 2 seconds, burst 30)
    let burn_governor = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(2)
        .burst_size(30)
        .use_headers()
        .finish()
        .unwrap();

    let create_routes = Router::new()
        .route("/api/secrets", post(crate::handlers::create::create_secret))
        .layer(
            GovernorLayer::new(create_governor)
                .error_handler(governor_error_handler),
        );

    let burn_routes = Router::new()
        .route(
            "/api/secrets/{id}",
            delete(crate::handlers::read::read_secret),
        )
        .layer(
            GovernorLayer::new(burn_governor)
                .error_handler(governor_error_handler),
        );

    Router::new()
        .route("/health", get(crate::handlers::health::health_check))
        .merge(create_routes)
        .merge(burn_routes)
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
