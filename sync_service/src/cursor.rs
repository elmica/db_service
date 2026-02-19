//! Read/write cursor file for incremental sync. Only advance after successful Convex send.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Cursor {
    /// Last OrderID we have fully synced (all related rows sent to Convex).
    pub last_order_id: i64,
}

impl Cursor {
    pub fn load(path: &Path) -> std::io::Result<Cursor> {
        let data = std::fs::read_to_string(path)?;
        let cursor: Cursor = serde_json::from_str(&data).unwrap_or(Cursor::default());
        Ok(cursor)
    }

    /// Load cursor; if file missing or invalid, return default (start from beginning).
    pub fn load_or_default(path: &Path) -> Cursor {
        Self::load(path).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, data)
    }
}
