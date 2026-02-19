//! Send order batches to Convex via HTTP. Expects Convex backend to expose an ingestion endpoint.

use crate::order_data::OrderBatch;
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum ConvexError {
    #[error("HTTP request: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Convex returned error status {0}: {1}")]
    Status(u16, String),
}

/// Send batch to Convex. On 4xx/5xx returns ConvexError so caller can retry (no cursor advance).
pub async fn send_batch(
    client: &Client,
    convex_url: &str,
    batch: &OrderBatch,
    api_key: Option<&str>,
) -> Result<(), ConvexError> {
    let mut req = client
        .post(convex_url)
        .json(batch)
        .timeout(Duration::from_secs(30));

    if let Some(key) = api_key {
        req = req.header("Authorization", format!("Bearer {}", key));
    }

    let res = req.send().await?;
    let status = res.status();
    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(ConvexError::Status(status.as_u16(), body));
    }
    Ok(())
}
