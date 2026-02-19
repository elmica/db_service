//! Load configuration from config file and environment.

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Path to the Aldelo Jet .mdb file (e.g. C:\Aldelo\Data\ChathamSandwich.mdb)
    pub mdb_path: PathBuf,
    /// Convex ingestion endpoint URL (e.g. https://your-deployment.convex.cloud/api/...)
    pub convex_url: String,
    /// Base poll interval in seconds (e.g. 5)
    #[serde(default = "default_poll_interval_secs")]
    pub poll_interval_secs: u64,
    /// Backoff: initial delay in seconds after error (e.g. 10)
    #[serde(default = "default_backoff_initial_secs")]
    pub backoff_initial_secs: u64,
    /// Backoff: multiplier (e.g. 2 for exponential)
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: u64,
    /// Backoff: max delay in seconds (e.g. 60)
    #[serde(default = "default_backoff_max_secs")]
    pub backoff_max_secs: u64,
    /// Path to cursor file (e.g. C:\AldeloSync\cursor.json)
    pub cursor_path: PathBuf,
    /// Number of quick retries on error before applying backoff (0–2)
    #[serde(default = "default_quick_retries")]
    pub quick_retries: u32,
    /// Batch size for reading and sending (e.g. 200)
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_poll_interval_secs() -> u64 {
    5
}
fn default_backoff_initial_secs() -> u64 {
    10
}
fn default_backoff_multiplier() -> u64 {
    2
}
fn default_backoff_max_secs() -> u64 {
    60
}
fn default_quick_retries() -> u32 {
    2
}
fn default_batch_size() -> usize {
    200
}

impl Config {
    /// Load from config file (e.g. config.toml) with env overrides.
    /// CONVEX_URL can be set via env; ALDELO_SYNC_* env vars override file values.
    pub fn load(config_path: Option<PathBuf>) -> Result<Self, config::ConfigError> {
        let mut c = config::Config::builder();

        if let Some(p) = config_path {
            if p.exists() {
                c = c.add_source(config::File::from(p));
            }
        }
        c = c.add_source(
            config::Environment::with_prefix("ALDELO_SYNC")
                .separator("__")
                .try_parsing(true),
        );

        let mut config: Config = c.build()?.try_deserialize()?;
        if let Ok(url) = std::env::var("CONVEX_URL") {
            config.convex_url = url;
        }
        Ok(config)
    }

    /// Poll interval as std::time::Duration
    pub fn poll_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.poll_interval_secs)
    }

    /// Initial backoff duration
    pub fn backoff_initial(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.backoff_initial_secs)
    }

    /// Max backoff duration
    pub fn backoff_max(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.backoff_max_secs)
    }
}
