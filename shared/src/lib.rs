use serde::{Deserialize, Serialize};

pub const MAX_PAYLOAD_BYTES: usize = 10 * 1024; // 10KB
pub const MIN_TTL_SECONDS: i64 = 300; // 5 minutes
pub const MAX_TTL_SECONDS: i64 = 86400; // 24 hours
pub const DEFAULT_TTL_SECONDS: i64 = 3600; // 1 hour

#[derive(Serialize, Deserialize)]
pub struct CreateSecretRequest {
    pub ciphertext: String,
    pub nonce: String,
    pub ttl_seconds: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateSecretResponse {
    pub id: String,
    pub expires_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReadSecretResponse {
    pub ciphertext: String,
    pub nonce: String,
}
