use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use std::str::FromStr;

/// Initializes a new SQLite connection pool with WAL mode enabled.
///
/// WAL (Write-Ahead Logging) mode is used for high-concurrency performance,
/// allowing multiple readers and one writer to operate simultaneously.
#[allow(dead_code)]
pub async fn init_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);

    // TODO: Configure pool settings like max_connections, idle_timeout, etc.
    let pool = SqlitePool::connect_with(options).await?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_init_pool_and_select_1() {
        // Use a temporary file to verify WAL mode, as :memory: doesn't use WAL
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let database_url = format!("sqlite:{}", temp_file.path().to_str().unwrap());

        let pool = init_pool(&database_url)
            .await
            .expect("Failed to initialize pool");

        let row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to execute query");

        assert_eq!(row.0, 1);

        // Verify WAL mode is enabled
        let journal_mode: (String,) = sqlx::query_as("PRAGMA journal_mode")
            .fetch_one(&pool)
            .await
            .expect("Failed to check journal mode");

        assert_eq!(journal_mode.0.to_lowercase(), "wal");
    }
}
