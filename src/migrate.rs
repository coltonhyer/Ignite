use sqlx::SqlitePool;
use tracing::info;

/// Runs all pending migrations in the migrations directory.
///
/// This is idempotent and safe to run on every boot.
#[allow(dead_code)]
pub async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    info!("Starting database migrations...");

    sqlx::migrate!("./migrations").run(pool).await?;

    info!("Database migrations completed successfully.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_run_migrations() {
        // Use an in-memory database for testing
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory database");

        // Run migrations
        run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        // Verify the 'secrets' table exists by querying it
        sqlx::query("SELECT id, ciphertext, nonce, expires_at, created_at FROM secrets")
            .fetch_all(&pool)
            .await
            .expect("Failed to query secrets table");

        // Verify the index exists
        let index_exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='index' AND name='idx_secrets_expires_at')"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check index existence");

        assert!(index_exists.0, "Index idx_secrets_expires_at should exist");

        // Run migrations again to ensure idempotency
        run_migrations(&pool)
            .await
            .expect("Failed to run migrations a second time");
    }
}
