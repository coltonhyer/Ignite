use std::net::SocketAddr;

use ignite::{db, migrate, router, workers};

use tokio_util::sync::CancellationToken;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🔥 Ignite — Burn After Reading");

    // Load configuration
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("Failed to parse PORT environment variable");

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "./ignite.db".to_string());

    // Initialize database pool
    let pool = db::init_pool(&database_url).await?;

    // Run migrations
    migrate::run_migrations(&pool).await?;

    // Create application router
    let app = router::create_router(pool.clone());

    // Setup cancellation token for graceful shutdown
    let cancel_token = CancellationToken::new();

    // Spawn the expiry worker
    let worker_handle = tokio::spawn(workers::expiry::spawn_expiry_worker(
        pool.clone(),
        cancel_token.clone(),
    ));

    // Start server
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    // Setup graceful shutdown handler for Axum
    let server_cancel_token = cancel_token.clone();
    let axum_shutdown = async move {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        info!("Received termination signal, starting graceful shutdown...");
        server_cancel_token.cancel();
    };

    if let Err(e) = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(axum_shutdown)
        .await
    {
        error!("Server error: {}", e);
    }

    info!("Waiting for background tasks to finish...");
    // Await the worker to ensure it cleanly shuts down
    let _ = worker_handle.await;

    info!("Graceful shutdown complete.");
    Ok(())
}
