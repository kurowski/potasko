//! VTODO conversion: parse and build iCalendar VTODO components.
//!
//! Maps between Task fields and iCalendar VTODO properties:
//! - SUMMARY ↔ title
//! - DESCRIPTION ↔ description
//! - DUE ↔ due_date
//! - PRIORITY ↔ priority (1-9)
//! - STATUS ↔ completed (COMPLETED vs NEEDS-ACTION)
//! - RRULE ↔ rrule
//! - UID ↔ uid

use chrono::{DateTime, Utc};
use icalendar::{Calendar, CalendarComponent, Component};
use thiserror::Error;

/// Parsed VTODO data extracted from an iCalendar string.
#[derive(Debug, Clone, Default)]
pub struct ParsedVTodo {
    pub uid: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub due: Option<DateTime<Utc>>,
    pub priority: Option<i32>,
    pub completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub rrule: Option<String>,
    /// Raw iCalendar string for round-trip preservation.
    #[allow(dead_code)]
    pub raw_icalendar: String,
}

impl ParsedVTodo {
    /// Get title with fallback to "Untitled".
    pub fn title(&self) -> &str {
        self.summary.as_deref().unwrap_or("Untitled")
    }

    /// Get due date as RFC3339 string.
    pub fn due_date_rfc3339(&self) -> Option<String> {
        self.due.map(|d| d.to_rfc3339())
    }

    /// Get completed_at as RFC3339 string.
    pub fn completed_at_rfc3339(&self) -> Option<String> {
        self.completed_at.map(|d| d.to_rfc3339())
    }
}

/// Errors that can occur during VTODO parsing or building.
#[derive(Debug, Error)]
pub enum VTodoError {
    #[error("No VTODO component found in iCalendar data")]
    NoVTodoFound,
    #[error("Failed to parse iCalendar: {0}")]
    ParseError(String),
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
}

/// Parse an iCalendar string and extract VTODO data.
///
/// Returns the first VTODO found in the calendar.
pub fn parse_vtodo(ical_str: &str) -> Result<ParsedVTodo, VTodoError> {
    let calendar = ical_str
        .parse::<Calendar>()
        .map_err(|e| VTodoError::ParseError(e.to_string()))?;

    // Find the first VTODO component
    for component in calendar.components {
        if let CalendarComponent::Todo(todo) = component {
            return Ok(extract_vtodo_data(&todo, ical_str.to_string()));
        }
    }

    Err(VTodoError::NoVTodoFound)
}

/// Extract data from a parsed Todo component.
fn extract_vtodo_data(todo: &icalendar::Todo, raw_icalendar: String) -> ParsedVTodo {
    let uid = todo.get_uid().map(|s| s.to_string());
    let summary = todo.get_summary().map(|s| s.to_string());
    let description = todo.get_description().map(|s| s.to_string());

    // Parse DUE date - use property getter since get_end() may not work for VTODO
    let due = get_property_value(todo, "DUE").and_then(|v| parse_ical_datetime(&v));

    // Parse PRIORITY (1-9, where 1 is highest)
    let priority = get_property_value(todo, "PRIORITY").and_then(|v| v.parse::<i32>().ok());

    // Parse STATUS - COMPLETED means done
    let status = get_property_value(todo, "STATUS");
    let completed = status.as_deref() == Some("COMPLETED");

    // Parse COMPLETED timestamp
    let completed_at = get_property_value(todo, "COMPLETED").and_then(|v| parse_ical_datetime(&v));

    // Parse RRULE
    let rrule = get_property_value(todo, "RRULE");

    ParsedVTodo {
        uid,
        summary,
        description,
        due,
        priority,
        completed,
        completed_at,
        rrule,
        raw_icalendar,
    }
}

/// Get a property value by name from a component.
fn get_property_value(todo: &icalendar::Todo, name: &str) -> Option<String> {
    todo.properties()
        .iter()
        .find(|(key, _)| key.as_str() == name)
        .map(|(_, prop)| prop.value().to_string())
}

/// Parse an iCalendar datetime string (e.g., "20250105T120000Z").
///
/// Handles three formats per RFC 5545:
/// - UTC time: "20250105T120000Z" (ends with Z)
/// - Local time: "20250105T120000" (no timezone, treated as UTC)
/// - Date only: "20250105" (treated as midnight UTC)
fn parse_ical_datetime(s: &str) -> Option<DateTime<Utc>> {
    // Try UTC format with literal Z suffix
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ") {
        return Some(DateTime::from_naive_utc_and_offset(naive, Utc));
    }
    // Try local time (no timezone) - treated as UTC
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%S") {
        return Some(DateTime::from_naive_utc_and_offset(naive, Utc));
    }
    // Try date only - treated as midnight UTC
    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y%m%d") {
        return date
            .and_hms_opt(0, 0, 0)
            .map(|naive| DateTime::from_naive_utc_and_offset(naive, Utc));
    }
    None
}

/// Data needed to build a VTODO.
pub struct VTodoBuildData {
    pub uid: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: Option<i32>,
    pub completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub rrule: Option<String>,
    /// Existing raw iCalendar to preserve unknown properties.
    pub existing_raw: Option<String>,
}

impl From<&crate::models::Task> for VTodoBuildData {
    fn from(task: &crate::models::Task) -> Self {
        Self {
            uid: task.uid.clone(),
            title: task.title.clone(),
            description: task.description.clone(),
            due_date: task.due_date,
            priority: task.priority,
            completed: task.completed,
            completed_at: task.completed_at,
            rrule: task.rrule.clone(),
            existing_raw: task.raw_icalendar.clone(),
        }
    }
}

/// Build an iCalendar string from task data.
///
/// If `existing_raw` is provided, preserves unknown properties from it.
pub fn build_vtodo(data: &VTodoBuildData) -> String {
    let mut todo = icalendar::Todo::new();

    // Required fields
    todo.uid(&data.uid);
    todo.summary(&data.title);

    // Add DTSTAMP (required by RFC 5545)
    todo.timestamp(Utc::now());

    // Optional fields
    if let Some(ref desc) = data.description {
        todo.description(desc);
    }

    if let Some(due) = data.due_date {
        todo.due(due);
    }

    if let Some(priority) = data.priority {
        todo.priority(priority as u32);
    }

    // Status
    if data.completed {
        todo.status(icalendar::TodoStatus::Completed);
        if let Some(completed_at) = data.completed_at {
            todo.completed(completed_at);
        } else {
            todo.completed(Utc::now());
        }
    } else {
        todo.status(icalendar::TodoStatus::NeedsAction);
    }

    // RRULE - add as raw property
    if let Some(ref rrule) = data.rrule {
        todo.add_property("RRULE", rrule);
    }

    // Preserve unknown properties from existing raw iCalendar
    if let Some(ref existing) = data.existing_raw {
        preserve_unknown_properties(&mut todo, existing);
    }

    // Build calendar wrapper
    let mut calendar = Calendar::new();
    calendar.push(todo);

    calendar.to_string()
}

/// Preserve unknown properties from an existing iCalendar string.
///
/// Adds properties from the original that we don't explicitly handle.
fn preserve_unknown_properties(todo: &mut icalendar::Todo, existing_raw: &str) {
    // Properties we explicitly handle - don't copy these
    const HANDLED_PROPERTIES: &[&str] = &[
        "UID",
        "SUMMARY",
        "DESCRIPTION",
        "DUE",
        "PRIORITY",
        "STATUS",
        "COMPLETED",
        "RRULE",
        "DTSTAMP",
        "CREATED",
        "LAST-MODIFIED",
        "SEQUENCE",
        "BEGIN",
        "END",
    ];

    // Parse existing calendar
    if let Ok(existing) = existing_raw.parse::<Calendar>() {
        for component in existing.components {
            if let CalendarComponent::Todo(existing_todo) = component {
                // Copy properties we don't handle
                for (key, prop) in existing_todo.properties() {
                    let key_str = key.as_str();
                    if !HANDLED_PROPERTIES.contains(&key_str) {
                        todo.add_property(key_str, prop.value());
                    }
                }
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_VTODO: &str = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//Test//EN
BEGIN:VTODO
UID:test-uid-123
DTSTAMP:20250105T120000Z
SUMMARY:Buy groceries
DESCRIPTION:Milk, eggs, bread
DUE:20250110T090000Z
PRIORITY:2
STATUS:NEEDS-ACTION
X-CUSTOM-PROP:custom-value
END:VTODO
END:VCALENDAR
"#;

    #[test]
    fn test_parse_vtodo() {
        let result = parse_vtodo(SAMPLE_VTODO).unwrap();

        assert_eq!(result.uid, Some("test-uid-123".to_string()));
        assert_eq!(result.summary, Some("Buy groceries".to_string()));
        assert_eq!(result.description, Some("Milk, eggs, bread".to_string()));
        assert_eq!(result.priority, Some(2));
        assert!(!result.completed);
        assert!(result.due.is_some());
    }

    #[test]
    fn test_parse_completed_vtodo() {
        let ical = r#"BEGIN:VCALENDAR
VERSION:2.0
BEGIN:VTODO
UID:completed-task
SUMMARY:Done task
STATUS:COMPLETED
COMPLETED:20250105T100000Z
END:VTODO
END:VCALENDAR
"#;

        let result = parse_vtodo(ical).unwrap();
        assert!(result.completed);
        assert!(result.completed_at.is_some());
    }

    #[test]
    fn test_build_vtodo() {
        let data = VTodoBuildData {
            uid: "new-task-456".to_string(),
            title: "Write tests".to_string(),
            description: Some("Unit tests for vtodo module".to_string()),
            due_date: Some(Utc::now()),
            priority: Some(1),
            completed: false,
            completed_at: None,
            rrule: None,
            existing_raw: None,
        };

        let ical = build_vtodo(&data);

        assert!(ical.contains("BEGIN:VCALENDAR"));
        assert!(ical.contains("BEGIN:VTODO"));
        assert!(ical.contains("UID:new-task-456"));
        assert!(ical.contains("SUMMARY:Write tests"));
        assert!(ical.contains("PRIORITY:1"));
        assert!(ical.contains("STATUS:NEEDS-ACTION"));
    }

    #[test]
    fn test_build_completed_vtodo() {
        let data = VTodoBuildData {
            uid: "completed-789".to_string(),
            title: "Completed task".to_string(),
            description: None,
            due_date: None,
            priority: None,
            completed: true,
            completed_at: Some(Utc::now()),
            rrule: None,
            existing_raw: None,
        };

        let ical = build_vtodo(&data);

        assert!(ical.contains("STATUS:COMPLETED"));
        assert!(ical.contains("COMPLETED:"));
    }

    #[test]
    fn test_round_trip_preserves_unknown_properties() {
        // Parse original with custom property
        let parsed = parse_vtodo(SAMPLE_VTODO).unwrap();

        // Build new VTODO using parsed data
        let data = VTodoBuildData {
            uid: parsed.uid.unwrap(),
            title: parsed.summary.unwrap(),
            description: parsed.description,
            due_date: parsed.due,
            priority: parsed.priority,
            completed: parsed.completed,
            completed_at: parsed.completed_at,
            rrule: parsed.rrule,
            existing_raw: Some(parsed.raw_icalendar),
        };

        let rebuilt = build_vtodo(&data);

        // Custom property should be preserved
        assert!(rebuilt.contains("X-CUSTOM-PROP:custom-value"));
    }

    #[test]
    fn test_vtodo_with_rrule() {
        let ical = r#"BEGIN:VCALENDAR
VERSION:2.0
BEGIN:VTODO
UID:recurring-task
SUMMARY:Weekly review
RRULE:FREQ=WEEKLY;BYDAY=MO
DUE:20250106T100000Z
STATUS:NEEDS-ACTION
END:VTODO
END:VCALENDAR
"#;

        let result = parse_vtodo(ical).unwrap();
        assert_eq!(result.rrule, Some("FREQ=WEEKLY;BYDAY=MO".to_string()));
    }

    #[test]
    fn test_datetime_parsing() {
        // UTC with Z suffix
        let parsed = super::parse_ical_datetime("20250110T090000Z");
        assert!(parsed.is_some(), "Should parse UTC time with Z");

        // Local time without timezone
        let parsed = super::parse_ical_datetime("20250110T090000");
        assert!(parsed.is_some(), "Should parse local time");

        // Date only
        let parsed = super::parse_ical_datetime("20250110");
        assert!(parsed.is_some(), "Should parse date-only");
    }

    #[test]
    fn test_no_vtodo_error() {
        let ical = r#"BEGIN:VCALENDAR
VERSION:2.0
BEGIN:VEVENT
UID:event-not-todo
SUMMARY:This is an event
END:VEVENT
END:VCALENDAR
"#;

        let result = parse_vtodo(ical);
        assert!(matches!(result, Err(VTodoError::NoVTodoFound)));
    }
}
