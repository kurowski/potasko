//! Output formatting for CLI (table and JSON).

use crate::models::{Account, Task, TaskList};
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

use super::args::OutputFormat;

/// Print tasks in the specified format.
pub fn print_tasks(tasks: &[Task], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(tasks).unwrap());
        }
        OutputFormat::Table => {
            if tasks.is_empty() {
                println!("No tasks found.");
                return;
            }

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec!["ID", "Title", "Due", "Priority", "Done", "List"]);

            for task in tasks {
                let done_cell = if task.completed {
                    Cell::new("Yes").fg(Color::Green)
                } else {
                    Cell::new("No")
                };

                let priority_cell = match task.priority {
                    Some(p) if p <= 3 => Cell::new(format!("{} (High)", p)).fg(Color::Red),
                    Some(p) if p <= 6 => Cell::new(format!("{} (Med)", p)).fg(Color::Yellow),
                    Some(p) => Cell::new(format!("{} (Low)", p)),
                    None => Cell::new("-"),
                };

                let due = task
                    .due_date
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "-".to_string());

                table.add_row(vec![
                    Cell::new(task.id),
                    Cell::new(&task.title),
                    Cell::new(due),
                    priority_cell,
                    done_cell,
                    Cell::new(task.list_id),
                ]);
            }

            println!("{table}");
        }
    }
}

/// Print a single task in the specified format.
pub fn print_task(task: &Task, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(task).unwrap());
        }
        OutputFormat::Table => {
            println!("Task #{}", task.id);
            println!("  Title:       {}", task.title);
            if let Some(desc) = &task.description {
                println!("  Description: {}", desc);
            }
            println!("  List ID:     {}", task.list_id);
            println!(
                "  Due:         {}",
                task.due_date
                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "-".to_string())
            );
            println!(
                "  Priority:    {}",
                task.priority
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "-".to_string())
            );
            println!("  Completed:   {}", if task.completed { "Yes" } else { "No" });
            if let Some(rrule) = &task.rrule {
                println!("  Recurrence:  {}", rrule);
            }
            println!(
                "  Created:     {}",
                task.created_at.format("%Y-%m-%d %H:%M")
            );
        }
    }
}

/// Print task lists in the specified format.
pub fn print_lists(lists: &[TaskList], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(lists).unwrap());
        }
        OutputFormat::Table => {
            if lists.is_empty() {
                println!("No lists found.");
                return;
            }

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec!["ID", "Name", "Color", "Account"]);

            for list in lists {
                table.add_row(vec![
                    Cell::new(list.id),
                    Cell::new(&list.name),
                    Cell::new(list.color.as_deref().unwrap_or("-")),
                    Cell::new(
                        list.account_id
                            .map(|id| id.to_string())
                            .unwrap_or_else(|| "Local".to_string()),
                    ),
                ]);
            }

            println!("{table}");
        }
    }
}

/// Print accounts in the specified format.
pub fn print_accounts(accounts: &[Account], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            // Don't include password in JSON output
            let safe: Vec<_> = accounts
                .iter()
                .map(|a| serde_json::json!({
                    "id": a.id,
                    "name": a.name,
                    "server_url": a.server_url,
                    "username": a.username,
                    "principal_url": a.principal_url,
                    "calendar_home_url": a.calendar_home_url,
                }))
                .collect();
            println!("{}", serde_json::to_string_pretty(&safe).unwrap());
        }
        OutputFormat::Table => {
            if accounts.is_empty() {
                println!("No accounts found.");
                return;
            }

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec!["ID", "Name", "Server", "Username"]);

            for account in accounts {
                table.add_row(vec![
                    Cell::new(account.id),
                    Cell::new(&account.name),
                    Cell::new(&account.server_url),
                    Cell::new(&account.username),
                ]);
            }

            println!("{table}");
        }
    }
}

/// Print success message.
pub fn print_success(message: &str) {
    println!("{}", message);
}

/// Print error message.
pub fn print_error(message: &str) {
    eprintln!("Error: {}", message);
}
