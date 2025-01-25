use crate::ServerInfo;
use reqwest;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
pub enum PostMethod {
    HTTPS,
    Disabled,
}

#[derive(Serialize)]
pub struct PostPayload {
    pub labels: HashMap<String, String>,
    pub result: ServerInfo,
}

pub async fn post_data(
    data: ServerInfo,
    labels: HashMap<String, String>,
    endpoint: &str,
    auth_token: Option<&str>,
    write_payload_to: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let payload = PostPayload {
        labels,
        result: data,
    };

    // Write payload to file if path is provided
    if let Some(path) = write_payload_to {
        match std::fs::write(path, serde_json::to_string_pretty(&payload)?) {
            Ok(_) => println!("Successfully saved payload to {}", path),
            Err(e) => eprintln!("Failed to write payload to {}: {}", path, e),
        }
    }

    // Validate endpoint when posting is enabled
    if endpoint.trim().is_empty() {
        return Err("Endpoint URL is required when --post is enabled".into());
    }

    let mut request = reqwest::Client::new().post(endpoint).json(&payload);

    if let Some(token) = auth_token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP request failed: {}", response.status()).into());
    }
    Ok(())
}
