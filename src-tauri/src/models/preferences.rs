//! Preference model for user settings.

use serde::{Deserialize, Serialize};

/// A single preference key-value pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}
