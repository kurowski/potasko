use serde::{Deserialize, Serialize};

/// Information about a CalDAV calendar discovered on the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarInfo {
    /// Relative or absolute URL path to the calendar collection.
    pub href: String,
    /// Display name (from DAV:displayname property).
    pub display_name: Option<String>,
    /// Color (from Apple's calendar-color property).
    pub color: Option<String>,
    /// Collection tag for change detection.
    pub ctag: Option<String>,
    /// Whether this calendar supports VTODO components.
    pub supports_vtodo: bool,
}

/// Result of testing a CalDAV account connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTestResult {
    /// Whether the connection test succeeded.
    pub success: bool,
    /// Discovered user principal URL.
    pub principal_url: Option<String>,
    /// Discovered calendar home URL.
    pub calendar_home_url: Option<String>,
    /// List of discovered calendars.
    pub calendars: Vec<CalendarInfo>,
    /// Error message if connection failed.
    pub error: Option<String>,
}

impl AccountTestResult {
    pub fn success(
        principal_url: String,
        calendar_home_url: String,
        calendars: Vec<CalendarInfo>,
    ) -> Self {
        Self {
            success: true,
            principal_url: Some(principal_url),
            calendar_home_url: Some(calendar_home_url),
            calendars,
            error: None,
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            principal_url: None,
            calendar_home_url: None,
            calendars: Vec::new(),
            error: Some(error.into()),
        }
    }
}
