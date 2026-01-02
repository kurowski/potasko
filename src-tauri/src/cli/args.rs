//! CLI argument definitions using clap.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "potasko")]
#[command(about = "Potasko task manager CLI", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Database path (defaults to ~/.local/share/com.vscode.potasko/potasko.db)
    #[arg(long, short)]
    pub database: Option<PathBuf>,

    /// Output format
    #[arg(long, short, default_value = "table", value_enum)]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Task operations
    Task {
        #[command(subcommand)]
        action: TaskCommands,
    },
    /// List operations
    List {
        #[command(subcommand)]
        action: ListCommands,
    },
    /// Account operations
    Account {
        #[command(subcommand)]
        action: AccountCommands,
    },
}

#[derive(Subcommand)]
pub enum TaskCommands {
    /// List tasks
    #[command(name = "list")]
    List {
        /// List ID (required unless using --today or --overdue)
        #[arg(long, short)]
        list: Option<i64>,

        /// Show only today's tasks (across all lists)
        #[arg(long, conflicts_with = "overdue")]
        today: bool,

        /// Show only overdue tasks (across all lists)
        #[arg(long, conflicts_with = "today")]
        overdue: bool,
    },
    /// Add a new task
    Add {
        /// Task title
        title: String,

        /// List ID
        #[arg(long, short)]
        list: i64,

        /// Due date (YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS)
        #[arg(long, short)]
        due: Option<String>,

        /// Priority (1-9, where 1 is highest)
        #[arg(long, short, value_parser = clap::value_parser!(i32).range(1..=9))]
        priority: Option<i32>,

        /// Description
        #[arg(long)]
        description: Option<String>,

        /// Recurrence (daily, weekly, monthly, yearly)
        #[arg(long, short)]
        recur: Option<String>,
    },
    /// Complete a task
    Complete {
        /// Task ID
        id: i64,
    },
    /// Delete a task
    Delete {
        /// Task ID
        id: i64,
    },
    /// Show task details
    Show {
        /// Task ID
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum ListCommands {
    /// List all task lists
    #[command(name = "list")]
    List,
    /// Add a new list
    Add {
        /// List name
        name: String,

        /// Color (hex code like #ff0000)
        #[arg(long, short)]
        color: Option<String>,
    },
    /// Delete a list
    Delete {
        /// List ID
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum AccountCommands {
    /// List all accounts
    #[command(name = "list")]
    List,
    /// Add a new CalDAV account
    Add {
        /// Account display name
        #[arg(long, short)]
        name: String,

        /// CalDAV server URL
        #[arg(long, short)]
        server: String,

        /// Username
        #[arg(long, short)]
        username: String,

        /// Password
        #[arg(long, short)]
        password: String,
    },
    /// Test connection to a CalDAV server
    Test {
        /// Account ID to test
        id: i64,
    },
    /// Delete an account
    Delete {
        /// Account ID
        id: i64,
    },
}
