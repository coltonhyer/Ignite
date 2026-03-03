mod db;
mod migrate;
mod router;

use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🔥 Ignite — Burn After Reading");

    // Load configuration
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "./ignite.db".to_string());

    // Initialize database pool
    let pool = db::init_pool(&database_url).await?;

    // Run migrations
    migrate::run_migrations(&pool).await?;

    // Create application router
    let app = router::create_router(pool);

    // Start server
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
