//! Command modules for CLI subcommands

pub mod agents;
pub mod projects;
pub mod tasks;
pub mod tenants;
pub mod workflows;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;

/// Create an HTTP client with optional API key authentication
pub fn create_client(api_key: Option<&str>) -> Result<Client> {
    let mut builder = Client::builder();

    if let Some(key) = api_key {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "X-API-Key",
            reqwest::header::HeaderValue::from_str(key).context("Invalid API key format")?,
        );
        builder = builder.default_headers(headers);
    }

    builder.build().context("Failed to create HTTP client")
}

/// Generic GET request helper
pub async fn get_json<T: DeserializeOwned>(client: &Client, url: &str) -> Result<T> {
    let resp = client.get(url).send().await.context("Request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Request failed with status {}: {}", status, body);
    }

    resp.json().await.context("Failed to parse response")
}

/// Generic POST request helper
pub async fn post_json<T: DeserializeOwned, B: serde::Serialize>(
    client: &Client,
    url: &str,
    body: &B,
) -> Result<T> {
    let resp = client
        .post(url)
        .json(body)
        .send()
        .await
        .context("Request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Request failed with status {}: {}", status, body);
    }

    resp.json().await.context("Failed to parse response")
}

/// Generic DELETE request helper
pub async fn delete(client: &Client, url: &str) -> Result<()> {
    let resp = client.delete(url).send().await.context("Request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Request failed with status {}: {}", status, body);
    }

    Ok(())
}
