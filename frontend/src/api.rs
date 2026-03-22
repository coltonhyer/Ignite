use shared::{CreateSecretRequest, CreateSecretResponse, ReadSecretResponse};

#[derive(serde::Deserialize)]
struct ApiError {
    error: String,
}

pub fn get_origin() -> String {
    #[cfg(target_arch = "wasm32")]
    return web_sys::window()
        .unwrap()
        .location()
        .origin()
        .unwrap_or_else(|_| "".into());
    #[cfg(not(target_arch = "wasm32"))]
    return "http://localhost:3000".to_string();
}

pub async fn create_secret(req: CreateSecretRequest) -> Result<CreateSecretResponse, String> {
    let client = reqwest::Client::new();
    let endpoint = format!("{}/api/secrets", get_origin());

    match client.post(&endpoint).json(&req).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                resp.json::<CreateSecretResponse>()
                    .await
                    .map_err(|_| "Invalid server response format.".to_string())
            } else {
                let status = resp.status();
                if let Ok(err_body) = resp.json::<ApiError>().await {
                    Err(err_body.error)
                } else {
                    Err(format!("Server returned error: {}", status))
                }
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

pub async fn burn_secret(id: &str) -> Result<ReadSecretResponse, String> {
    let client = reqwest::Client::new();
    let endpoint = format!("{}/api/secrets/{}", get_origin(), id);

    match client.delete(&endpoint).send().await {
        Ok(resp) => {
            if resp.status() == 410 {
                return Err("Already viewed. Destroyed from server.".to_string());
            }
            if resp.status().is_success() {
                resp.json::<ReadSecretResponse>()
                    .await
                    .map_err(|_| "Invalid server response format.".to_string())
            } else {
                let status = resp.status();
                if let Ok(err_body) = resp.json::<ApiError>().await {
                    Err(err_body.error)
                } else {
                    Err(format!("Server returned error: {}", status))
                }
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}
