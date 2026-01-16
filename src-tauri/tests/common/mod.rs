//! Test infrastructure for E2E sync tests.
//!
//! Provides helpers for:
//! - Creating isolated test databases
//! - Setting up Radicale calendars
//! - CLI command execution with assert_cmd

use assert_cmd::Command;
use reqwest::blocking::Client;
use std::path::PathBuf;
use std::sync::Once;
use tempfile::TempDir;
use uuid::Uuid;

// Install ring crypto provider once for all tests (required for rustls-no-provider)
static CRYPTO_PROVIDER: Once = Once::new();

fn ensure_crypto_provider() {
    CRYPTO_PROVIDER.call_once(|| {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install crypto provider");
    });
}

/// Radicale server configuration for tests.
pub const RADICALE_URL: &str = "http://localhost:5232";
pub const RADICALE_USER: &str = "test";
pub const RADICALE_PASS: &str = "test";

/// Test context holding temp database and unique calendar name.
pub struct TestContext {
    pub db_dir: TempDir,
    pub db_path: PathBuf,
    pub calendar_name: String,
    pub calendar_url: String,
}

impl TestContext {
    /// Create a new test context with isolated database and unique calendar.
    pub fn new() -> Self {
        // Ensure crypto provider is installed before creating any HTTP clients
        ensure_crypto_provider();

        let db_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = db_dir.path().join("test.db");
        let calendar_name = format!("test-{}", Uuid::new_v4());
        let calendar_url = format!("{}/{}/{}/", RADICALE_URL, RADICALE_USER, calendar_name);

        Self {
            db_dir,
            db_path,
            calendar_name,
            calendar_url,
        }
    }

    /// Get a CLI command configured with this test's database.
    pub fn cli(&self) -> Command {
        let mut cmd = Command::cargo_bin("potasko").expect("Failed to find potasko binary");
        cmd.arg("--database").arg(&self.db_path);
        cmd
    }

    /// Create the calendar on Radicale.
    pub fn create_calendar(&self) -> Result<(), String> {
        let client = Client::new();
        let response = client
            .request(
                reqwest::Method::from_bytes(b"MKCALENDAR").unwrap(),
                &self.calendar_url,
            )
            .basic_auth(RADICALE_USER, Some(RADICALE_PASS))
            .send()
            .map_err(|e| e.to_string())?;

        if response.status().is_success() || response.status() == 405 {
            // 405 means calendar already exists, which is fine
            Ok(())
        } else {
            Err(format!(
                "Failed to create calendar: {} {}",
                response.status(),
                response.text().unwrap_or_default()
            ))
        }
    }

    /// Delete the calendar on Radicale (cleanup).
    pub fn delete_calendar(&self) -> Result<(), String> {
        let client = Client::new();
        let response = client
            .delete(&self.calendar_url)
            .basic_auth(RADICALE_USER, Some(RADICALE_PASS))
            .send()
            .map_err(|e| e.to_string())?;

        if response.status().is_success() || response.status() == 404 {
            Ok(())
        } else {
            Err(format!(
                "Failed to delete calendar: {} {}",
                response.status(),
                response.text().unwrap_or_default()
            ))
        }
    }

    /// Add an account via CLI and return its ID.
    pub fn add_account(&self) -> Result<i64, String> {
        let output = self
            .cli()
            .args([
                "account",
                "add",
                "--name",
                "Test",
                "--server",
                RADICALE_URL,
                "--username",
                RADICALE_USER,
                "--password",
                RADICALE_PASS,
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(format!(
                "Failed to add account: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Account ID is typically 1 for first account
        Ok(1)
    }

    /// Sync the account - discovers calendars and imports them as lists.
    /// Returns the ID of the imported list.
    pub fn sync_account(&self, account_id: i64) -> Result<i64, String> {
        let output = self
            .cli()
            .args(["sync", "account", &account_id.to_string()])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(format!(
                "Failed to sync account: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Get the list ID for the imported calendar
        self.get_synced_list_id(account_id)
    }

    /// Get the list ID for this test's specific calendar.
    pub fn get_synced_list_id(&self, _account_id: i64) -> Result<i64, String> {
        // Query the database for the list matching this test's calendar_url
        let output = std::process::Command::new("sqlite3")
            .arg(&self.db_path)
            .arg(format!(
                "SELECT id FROM task_lists WHERE caldav_url = '{}' LIMIT 1;",
                self.calendar_url
            ))
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(format!(
                "Failed to query list: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let list_id: i64 = stdout
            .trim()
            .parse()
            .map_err(|_| format!("No list found for calendar URL {}", self.calendar_url))?;

        Ok(list_id)
    }

    /// Create a VTODO on the server and return its URL.
    pub fn create_server_vtodo(&self, uid: &str, summary: &str) -> Result<String, String> {
        let vtodo = format!(
            "BEGIN:VCALENDAR\r\n\
             VERSION:2.0\r\n\
             PRODID:-//Potasko Test//EN\r\n\
             BEGIN:VTODO\r\n\
             UID:{}\r\n\
             SUMMARY:{}\r\n\
             STATUS:NEEDS-ACTION\r\n\
             END:VTODO\r\n\
             END:VCALENDAR\r\n",
            uid, summary
        );

        let task_url = format!("{}{}.ics", self.calendar_url, uid);

        let client = Client::new();
        let response = client
            .put(&task_url)
            .basic_auth(RADICALE_USER, Some(RADICALE_PASS))
            .header("Content-Type", "text/calendar; charset=utf-8")
            .body(vtodo)
            .send()
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            Ok(task_url)
        } else {
            Err(format!(
                "Failed to create VTODO: {} {}",
                response.status(),
                response.text().unwrap_or_default()
            ))
        }
    }

    /// Get a VTODO from the server.
    pub fn get_server_vtodo(&self, uid: &str) -> Result<Option<String>, String> {
        let task_url = format!("{}{}.ics", self.calendar_url, uid);

        let client = Client::new();
        let response = client
            .get(&task_url)
            .basic_auth(RADICALE_USER, Some(RADICALE_PASS))
            .send()
            .map_err(|e| e.to_string())?;

        if response.status() == 404 {
            Ok(None)
        } else if response.status().is_success() {
            Ok(Some(response.text().map_err(|e| e.to_string())?))
        } else {
            Err(format!(
                "Failed to get VTODO: {} {}",
                response.status(),
                response.text().unwrap_or_default()
            ))
        }
    }

    /// Update a VTODO on the server.
    pub fn update_server_vtodo(&self, uid: &str, summary: &str) -> Result<(), String> {
        let vtodo = format!(
            "BEGIN:VCALENDAR\r\n\
             VERSION:2.0\r\n\
             PRODID:-//Potasko Test//EN\r\n\
             BEGIN:VTODO\r\n\
             UID:{}\r\n\
             SUMMARY:{}\r\n\
             STATUS:NEEDS-ACTION\r\n\
             END:VTODO\r\n\
             END:VCALENDAR\r\n",
            uid, summary
        );

        let task_url = format!("{}{}.ics", self.calendar_url, uid);

        let client = Client::new();
        let response = client
            .put(&task_url)
            .basic_auth(RADICALE_USER, Some(RADICALE_PASS))
            .header("Content-Type", "text/calendar; charset=utf-8")
            .body(vtodo)
            .send()
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!(
                "Failed to update VTODO: {} {}",
                response.status(),
                response.text().unwrap_or_default()
            ))
        }
    }

    /// Delete a VTODO from the server.
    pub fn delete_server_vtodo(&self, uid: &str) -> Result<(), String> {
        let task_url = format!("{}{}.ics", self.calendar_url, uid);

        let client = Client::new();
        let response = client
            .delete(&task_url)
            .basic_auth(RADICALE_USER, Some(RADICALE_PASS))
            .send()
            .map_err(|e| e.to_string())?;

        if response.status().is_success() || response.status() == 404 {
            Ok(())
        } else {
            Err(format!(
                "Failed to delete VTODO: {} {}",
                response.status(),
                response.text().unwrap_or_default()
            ))
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Clean up calendar on Radicale
        let _ = self.delete_calendar();
    }
}
