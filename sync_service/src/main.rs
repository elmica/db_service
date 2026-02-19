//! Aldelo Jet → Convex sync service. Run as Windows service via NSSM.
//! Read-only MDB access; incremental sync with cursor file; exponential backoff on errors.

mod backoff;
mod config;
mod convex;
mod cursor;
mod mdb;
mod order_data;

use crate::backoff::BackoffState;
use crate::config::Config;
use crate::cursor::Cursor;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let config_path = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .or_else(|| Some(std::path::PathBuf::from("config.toml")));
    let config = match Config::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    let cursor_path = config.cursor_path.clone();
    let mut cursor = Cursor::load_or_default(&cursor_path);
    let mut backoff = BackoffState::new(&config);
    let client = Arc::new(
        reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("HTTP client"),
    );
    let api_key = std::env::var("CONVEX_API_KEY").ok();

    info!(
        "Starting sync: mdb={}, cursor at {}",
        config.mdb_path.display(),
        config.cursor_path.display()
    );
    info!("Base poll interval: {}s", config.poll_interval_secs);

    loop {
        let interval = backoff.current_interval();
        info!("Sleeping {:?} until next poll", interval);
        tokio::time::sleep(interval).await;

        let mut quick_retries_left = config.quick_retries;
        let mut backoff_applied = false;

        loop {
            match run_one_poll(&config, &client, &api_key, &mut cursor, &cursor_path).await {
                Ok(()) => {
                    backoff.on_success();
                    break;
                }
                Err(e) => {
                    if quick_retries_left > 0 {
                        quick_retries_left -= 1;
                        warn!(
                            "Poll failed ({} quick retries left): {}",
                            quick_retries_left, e
                        );
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    } else {
                        error!("Poll failed after quick retries: {}", e);
                        backoff.on_error();
                        backoff_applied = true;
                        break;
                    }
                }
            }
        }

        if backoff_applied {
            warn!("Next poll in {:?} (backoff)", backoff.current_interval());
        }
    }
}

async fn run_one_poll(
    config: &Config,
    client: &reqwest::Client,
    api_key: &Option<String>,
    cursor: &mut Cursor,
    cursor_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let batch = crate::mdb::read_next_batch(
        &config.mdb_path,
        cursor.last_order_id,
        config.batch_size,
    )?;

    if batch.is_empty() {
        return Ok(());
    }

    let key = api_key.as_deref();
    crate::convex::send_batch(client, &config.convex_url, &batch, key).await?;

    if let Some(max_id) = batch.max_order_id() {
        cursor.last_order_id = max_id;
        cursor.save(cursor_path)?;
        info!(
            "Synced {} headers, {} transactions, {} payments, {} refunds; cursor -> {}",
            batch.headers.len(),
            batch.transactions.len(),
            batch.payments.len(),
            batch.refunds.len(),
            max_id
        );
    }

    Ok(())
}
