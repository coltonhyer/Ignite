use sqlx::SqlitePool;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

const POLL_INTERVAL_SECS: u64 = 60;

pub async fn spawn_expiry_worker(pool: SqlitePool, cancel: CancellationToken) {
    info!("Starting TTL expiry background worker");

    let mut interval = tokio::time::interval(Duration::from_secs(POLL_INTERVAL_SECS));

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                info!("Stopping TTL expiry background worker");
                break;
            }
            _ = interval.tick() => {
                match sqlx::query!(
                    r#"
                    DELETE FROM secrets
                    WHERE expires_at < datetime('now')
                    "#
                )
                .execute(&pool)
                .await
                {
                    Ok(result) => {
                        let rows_affected = result.rows_affected();
                        info!("Purged {} expired secrets", rows_affected);
                    }
                    Err(e) => {
                        error!("Failed to purge expired secrets: {}", e);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::time::Duration;
    use tokio::time::sleep;

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

    #[tokio::test]
    async fn test_expiry_worker() {
        let pool = setup_db().await;

        let data1 = b"data1".to_vec();
        let nonce1 = b"nonce1".to_vec();
        // Insert a secret that expires in the past
        sqlx::query!(
            "INSERT INTO secrets (id, ciphertext, nonce, expires_at) VALUES (?, ?, ?, datetime('now', '-1 day'))",
            "1",
            data1,
            nonce1
        )
        .execute(&pool)
        .await
        .unwrap();

        let data2 = b"data2".to_vec();
        let nonce2 = b"nonce2".to_vec();
        // Insert a secret that expires in the future
        sqlx::query!(
            "INSERT INTO secrets (id, ciphertext, nonce, expires_at) VALUES (?, ?, ?, datetime('now', '+1 day'))",
            "2",
            data2,
            nonce2
        )
        .execute(&pool)
        .await
        .unwrap();

        // Verify there are 2 secrets initially
        let count: i64 = sqlx::query_scalar!("SELECT count(*) FROM secrets")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2);

        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();

        // Use a smaller poll interval for the test to run quickly
        // We override interval inside test scope if we could, but interval is hardcoded in the function.
        // Let's spawn the worker. It will immediately tick once.
        let worker_handle = tokio::spawn(spawn_expiry_worker(pool.clone(), cancel_clone));

        // Yield to let the worker run its first tick
        sleep(Duration::from_millis(100)).await;

        // Cancel worker
        cancel.cancel();

        // Wait for worker to finish
        let _ = worker_handle.await;

        // Verify only 1 secret is left (the one in the future)
        let count: i64 = sqlx::query_scalar!("SELECT count(*) FROM secrets")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);

        // Verify the remaining secret is the one we expect
        let remaining_id: String = sqlx::query_scalar!("SELECT id FROM secrets")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(remaining_id, "2");
    }
}
