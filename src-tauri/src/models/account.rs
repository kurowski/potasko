use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A CalDAV account (server connection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Database row ID.
    pub id: i64,
    /// Display name for the account.
    pub name: String,
    /// CalDAV server base URL.
    pub server_url: String,
    pub username: String,
    /// Password stored in plaintext for now.
    /// TODO: Use tauri-plugin-stronghold or system keyring before release.
    pub password: String,

    // Discovered CalDAV endpoints
    /// User's principal URL (discovered via well-known or PROPFIND).
    pub principal_url: Option<String>,
    /// Calendar home URL (where calendars are listed).
    pub calendar_home_url: Option<String>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccount {
    pub name: String,
    pub server_url: String,
    pub username: String,
    pub password: String,
    /// Pre-discovered principal URL (from connection test).
    pub principal_url: Option<String>,
    /// Pre-discovered calendar home URL (from connection test).
    pub calendar_home_url: Option<String>,
}

/// Data for updating an existing account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccount {
    pub name: Option<String>,
    pub server_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub principal_url: Option<String>,
    pub calendar_home_url: Option<String>,
}
