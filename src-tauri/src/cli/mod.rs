//! CLI module for potasko-cli binary.

pub mod args;
pub mod output;

use std::path::PathBuf;

/// Get the default database path based on platform conventions.
pub fn default_db_path() -> PathBuf {
    let data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));

    #[cfg(target_os = "linux")]
    let app_dir = data_dir.join("potasko");

    #[cfg(target_os = "macos")]
    let app_dir = data_dir.join("com.potasko.app");

    #[cfg(target_os = "windows")]
    let app_dir = data_dir.join("Potasko");

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let app_dir = data_dir.join("potasko");

    app_dir.join("potasko.db")
}
