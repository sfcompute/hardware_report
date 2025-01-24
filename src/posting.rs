use reqwest;
use serde::Serialize;
use std::error::Error;

#[derive(Debug)]
pub enum PostMethod {
    HTTPS,
    Disabled,
}

pub async fn post_data<T: Serialize>(
    data: &T,
    endpoint: &str,
    auth_token: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let mut request = reqwest::Client::new().post(endpoint).json(data);

    if let Some(token) = auth_token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP request failed: {}", response.status()).into());
    }
    Ok(())
}

#[allow(dead_code)]
async fn post_https<T: Serialize>(
    data: &T,
    endpoint: &str,
    auth_token: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let mut request = reqwest::Client::new().post(endpoint).json(data);

    if let Some(token) = auth_token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP request failed: {}", response.status()).into());
    }
    Ok(())
}
