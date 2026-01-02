//! Core business logic module.
//!
//! Contains pure functions that can be called from both CLI and Tauri commands.
//! All functions take `&SqlitePool` directly instead of Tauri-specific state wrappers.

pub mod accounts;
pub mod error;
pub mod lists;
pub mod tasks;

pub use error::CoreError;
