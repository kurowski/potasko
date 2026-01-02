use thiserror::Error;

/// Shared error type for core business logic.
/// Used by both CLI and Tauri commands.
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("CalDAV error: {0}")]
    CalDav(String),
}

/// For easy conversion to String in Tauri commands
impl From<CoreError> for String {
    fn from(e: CoreError) -> String {
        e.to_string()
    }
}
