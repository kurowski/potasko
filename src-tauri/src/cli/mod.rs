//! CLI module for potasko binary.

pub mod args;
pub mod output;

use std::path::PathBuf;

/// Tauri app identifier (must match tauri.conf.json)
const APP_IDENTIFIER: &str = "net.kurowski.potasko";

/// Get the default database path matching Tauri's app data directory.
pub fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_IDENTIFIER)
        .join("potasko.db")
}
