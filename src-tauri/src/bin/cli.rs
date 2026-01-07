//! Potasko CLI binary.
//!
//! A command-line interface for testing and managing Potasko tasks without the GUI.

use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use clap::Parser;
use potasko_lib::cli::args::{AccountCommands, Cli, Commands, ListCommands, SyncCommands, TaskCommands};
use potasko_lib::cli::output::{
    print_accounts, print_error, print_lists, print_success, print_task, print_tasks,
};
use potasko_lib::cli::default_db_path;
use potasko_lib::core;
use potasko_lib::{CreateAccount, CreateTask, CreateTaskList};
use sqlx::SqlitePool;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    // Determine database path
    let db_path = cli.database.unwrap_or_else(default_db_path);

    // Check if database exists
    if !db_path.exists() {
        // Try to create parent directories
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    print_error(&format!("Failed to create database directory: {}", e));
                    return ExitCode::FAILURE;
                }
            }
        }
    }

    // Initialize database
    let pool = match potasko_lib::db::init(&db_path).await {
        Ok(pool) => pool,
        Err(e) => {
            print_error(&format!("Failed to initialize database: {}", e));
            return ExitCode::FAILURE;
        }
    };

    // Handle command
    match run_command(cli.command, cli.format, &pool).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            print_error(&e);
            ExitCode::FAILURE
        }
    }
}

async fn run_command(
    command: Commands,
    format: potasko_lib::cli::args::OutputFormat,
    pool: &SqlitePool,
) -> Result<(), String> {
    match command {
        Commands::Task { action } => handle_task_command(action, format, pool).await,
        Commands::List { action } => handle_list_command(action, format, pool).await,
        Commands::Account { action } => handle_account_command(action, format, pool).await,
        Commands::Sync { action } => handle_sync_command(action, format, pool).await,
    }
}

async fn handle_task_command(
    action: TaskCommands,
    format: potasko_lib::cli::args::OutputFormat,
    pool: &SqlitePool,
) -> Result<(), String> {
    match action {
        TaskCommands::List { list, today, overdue } => {
            let tasks = if today {
                core::tasks::get_tasks_today(pool).await
            } else if overdue {
                core::tasks::get_tasks_overdue(pool).await
            } else if let Some(list_id) = list {
                core::tasks::get_tasks(list_id, pool).await
            } else {
                return Err("Please specify --list ID, --today, or --overdue".to_string());
            };

            match tasks {
                Ok(tasks) => {
                    print_tasks(&tasks, format);
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            }
        }

        TaskCommands::Add {
            title,
            list,
            due,
            priority,
            description,
            recur,
        } => {
            let due_date = if let Some(due_str) = due {
                Some(parse_due_date(&due_str)?)
            } else {
                None
            };

            let rrule = recur.map(|r| recurrence_to_rrule(&r));

            let data = CreateTask {
                list_id: list,
                title,
                description,
                due_date,
                priority,
                rrule,
            };

            match core::tasks::create_task(data, pool).await {
                Ok(task) => {
                    print_success(&format!("Created task #{}", task.id));
                    print_task(&task, format);
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            }
        }

        TaskCommands::Complete { id } => {
            match core::tasks::toggle_task_completion(id, pool).await {
                Ok(task) => {
                    if task.completed {
                        print_success(&format!("Completed task #{}", id));
                    } else {
                        print_success(&format!("Reopened task #{}", id));
                    }
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            }
        }

        TaskCommands::Delete { id } => match core::tasks::delete_task(id, pool).await {
            Ok(()) => {
                print_success(&format!("Deleted task #{}", id));
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        },

        TaskCommands::Show { id } => match core::tasks::get_task(id, pool).await {
            Ok(task) => {
                print_task(&task, format);
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        },
    }
}

async fn handle_list_command(
    action: ListCommands,
    format: potasko_lib::cli::args::OutputFormat,
    pool: &SqlitePool,
) -> Result<(), String> {
    match action {
        ListCommands::List => match core::lists::get_lists(pool).await {
            Ok(lists) => {
                print_lists(&lists, format);
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        },

        ListCommands::Add { name, color } => {
            let data = CreateTaskList { name, color };

            match core::lists::create_list(data, pool).await {
                Ok(list) => {
                    print_success(&format!("Created list #{}: {}", list.id, list.name));
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            }
        }

        ListCommands::Delete { id } => match core::lists::delete_list(id, pool).await {
            Ok(()) => {
                print_success(&format!("Deleted list #{}", id));
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        },
    }
}

async fn handle_account_command(
    action: AccountCommands,
    format: potasko_lib::cli::args::OutputFormat,
    pool: &SqlitePool,
) -> Result<(), String> {
    match action {
        AccountCommands::List => match core::accounts::get_accounts(pool).await {
            Ok(accounts) => {
                print_accounts(&accounts, format);
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        },

        AccountCommands::Add {
            name,
            server,
            username,
            password,
        } => {
            // Test connection first
            print_success("Testing connection...");
            let test_result =
                core::accounts::test_account_connection(&server, &username, &password).await;

            let (principal_url, calendar_home_url) = match test_result {
                Ok(result) if result.success => {
                    print_success("Connection successful!");
                    if !result.calendars.is_empty() {
                        print_success(&format!("Found {} calendar(s)", result.calendars.len()));
                    }
                    (result.principal_url, result.calendar_home_url)
                }
                Ok(result) => {
                    return Err(format!(
                        "Connection failed: {}",
                        result.error.unwrap_or_else(|| "Unknown error".to_string())
                    ));
                }
                Err(e) => {
                    return Err(format!("Connection test error: {}", e));
                }
            };

            let data = CreateAccount {
                name,
                server_url: server,
                username,
                password,
                principal_url,
                calendar_home_url,
            };

            match core::accounts::create_account(data, pool).await {
                Ok(account) => {
                    print_success(&format!("Created account #{}: {}", account.id, account.name));
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            }
        }

        AccountCommands::Test { id } => {
            let account = core::accounts::get_account(id, pool)
                .await
                .map_err(|e| e.to_string())?;

            print_success(&format!("Testing connection for account '{}'...", account.name));

            match core::accounts::test_account_connection(
                &account.server_url,
                &account.username,
                &account.password,
            )
            .await
            {
                Ok(result) => {
                    if result.success {
                        print_success("Connection successful!");
                        if let Some(url) = result.principal_url {
                            print_success(&format!("Principal URL: {}", url));
                        }
                        if let Some(url) = result.calendar_home_url {
                            print_success(&format!("Calendar home: {}", url));
                        }
                        if !result.calendars.is_empty() {
                            print_success(&format!("Found {} calendar(s)", result.calendars.len()));
                            for cal in &result.calendars {
                                let name = cal.display_name.as_deref().unwrap_or("<unnamed>");
                                print_success(&format!("  - {}", name));
                            }
                        }
                    } else {
                        return Err(format!(
                            "Connection failed: {}",
                            result.error.unwrap_or_else(|| "Unknown error".to_string())
                        ));
                    }
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            }
        }

        AccountCommands::Delete { id } => match core::accounts::delete_account(id, pool).await {
            Ok(()) => {
                print_success(&format!("Deleted account #{}", id));
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        },
    }
}

/// Parse a due date string into DateTime<Utc>.
/// Supports formats: YYYY-MM-DD, YYYY-MM-DDTHH:MM:SS, YYYY-MM-DD HH:MM:SS
fn parse_due_date(s: &str) -> Result<chrono::DateTime<Utc>, String> {
    // Try date-only format first (YYYY-MM-DD) - default to midnight UTC
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap()));
    }

    // Try ISO 8601 format (YYYY-MM-DDTHH:MM:SS)
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(Utc.from_utc_datetime(&dt));
    }

    // Try space-separated format (YYYY-MM-DD HH:MM:SS)
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(Utc.from_utc_datetime(&dt));
    }

    // Try simpler format (YYYY-MM-DD HH:MM)
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
        return Ok(Utc.from_utc_datetime(&dt));
    }

    Err(format!(
        "Invalid date format '{}'. Expected YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS",
        s
    ))
}

/// Convert simple recurrence strings to RRULE format.
fn recurrence_to_rrule(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "daily" | "day" | "d" => "FREQ=DAILY".to_string(),
        "weekly" | "week" | "w" => "FREQ=WEEKLY".to_string(),
        "monthly" | "month" | "m" => "FREQ=MONTHLY".to_string(),
        "yearly" | "year" | "y" => "FREQ=YEARLY".to_string(),
        // If already in RRULE format, pass through
        other if other.starts_with("FREQ=") => other.to_uppercase(),
        other => format!("FREQ={}", other.to_uppercase()),
    }
}

async fn handle_sync_command(
    action: SyncCommands,
    _format: potasko_lib::cli::args::OutputFormat,
    pool: &SqlitePool,
) -> Result<(), String> {
    use potasko_lib::sync::SyncEngine;

    let engine = SyncEngine::new(pool.clone());

    match action {
        SyncCommands::List { id } => {
            print_success(&format!("Syncing list #{}...", id));
            let result = engine.sync_list(id).await;
            print_sync_result(&result);
            if result.success {
                Ok(())
            } else {
                Err(result.error.unwrap_or_else(|| "Sync failed".to_string()))
            }
        }

        SyncCommands::Download { id } => {
            print_success(&format!("Downloading tasks for list #{}...", id));
            let result = engine.initial_download(id).await;
            print_sync_result(&result);
            if result.success {
                Ok(())
            } else {
                Err(result.error.unwrap_or_else(|| "Download failed".to_string()))
            }
        }

        SyncCommands::Account { id } => {
            print_success(&format!("Syncing account #{}...", id));

            let result = engine.sync_account(id).await;

            // Print calendar discovery results
            if result.calendars_imported > 0 {
                print_success(&format!("Imported {} new calendar(s)", result.calendars_imported));
            }

            // Print results for each list
            if result.list_results.is_empty() && result.calendars_imported == 0 {
                print_success("No calendars found for this account");
            } else {
                for list_result in &result.list_results {
                    print_sync_result(list_result);
                }
            }

            if result.success {
                Ok(())
            } else {
                Err(result.error.unwrap_or_else(|| "Some lists failed to sync".to_string()))
            }
        }

        SyncCommands::Status { id } => {
            let row = sqlx::query!(
                r#"
                SELECT
                    l.name,
                    l.caldav_url,
                    l.ctag,
                    l.updated_at,
                    (SELECT COUNT(*) FROM tasks t
                     WHERE t.list_id = l.id
                     AND (t.local_version > t.synced_version OR t.sync_status != 'synced')) as pending_count
                FROM task_lists l
                WHERE l.id = ?
                "#,
                id
            )
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("List #{} not found", id))?;

            println!("List: {}", row.name);
            println!("CalDAV URL: {}", row.caldav_url.as_deref().unwrap_or("(none)"));
            println!("CTag: {}", row.ctag.as_deref().unwrap_or("(none)"));
            println!("Last updated: {}", row.updated_at);
            println!("Pending changes: {}", row.pending_count);

            Ok(())
        }
    }
}

fn print_sync_result(result: &potasko_lib::sync::SyncResult) {
    let s = &result.stats;
    if result.success {
        let mut parts = Vec::new();
        if s.pushed_created > 0 { parts.push(format!("pushed {} new", s.pushed_created)); }
        if s.pushed_updated > 0 { parts.push(format!("pushed {} updated", s.pushed_updated)); }
        if s.pushed_deleted > 0 { parts.push(format!("pushed {} deleted", s.pushed_deleted)); }
        if s.pulled_created > 0 { parts.push(format!("pulled {} new", s.pulled_created)); }
        if s.pulled_updated > 0 { parts.push(format!("pulled {} updated", s.pulled_updated)); }
        if s.pulled_deleted > 0 { parts.push(format!("pulled {} deleted", s.pulled_deleted)); }
        if s.conflicts > 0 { parts.push(format!("{} conflicts", s.conflicts)); }

        if parts.is_empty() {
            print_success("Sync complete (no changes)");
        } else {
            print_success(&format!("Sync complete: {}", parts.join(", ")));
        }
    } else {
        print_error(&format!("Sync failed: {}", result.error.as_deref().unwrap_or("Unknown error")));
    }

    for err in &s.errors {
        print_error(&format!("  - {}", err));
    }
}
