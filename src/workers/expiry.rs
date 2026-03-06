use sqlx::SqlitePool;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub async fn spawn_expiry_worker(pool: SqlitePool, cancel: CancellationToken) {
    info!("Starting TTL expiry background worker");

    let mut interval = tokio::time::interval(Duration::from_secs(60));

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
