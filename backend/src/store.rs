use sqlx::SqlitePool;

#[derive(Clone)]
pub struct SecretStore {
    pub pool: SqlitePool,
}

pub struct SecretRow {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl SecretStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn is_alive(&self) -> Result<(), sqlx::Error> {
        sqlx::query!("SELECT 1 as is_alive")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_secret(
        &self,
        id: &str,
        ciphertext: &[u8],
        nonce: &[u8],
        expires_at: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO secrets (id, ciphertext, nonce, expires_at)
            VALUES (?, ?, ?, ?)
            "#,
            id,
            ciphertext,
            nonce,
            expires_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn burn_secret(&self, id: &str) -> Result<Option<SecretRow>, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM secrets
            WHERE id = ?1 AND expires_at > datetime('now')
            RETURNING ciphertext, nonce
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| SecretRow {
            ciphertext: row.ciphertext,
            nonce: row.nonce,
        }))
    }

    pub async fn purge_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM secrets
            WHERE expires_at < datetime('now')
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
