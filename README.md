# Potasko

An offline-first CalDAV task manager built with Tauri 2.0, Rust, and SvelteKit.

## Development (Devcontainer)

This project uses a devcontainer with all dependencies pre-installed. Open in VS Code with the Dev Containers extension or use GitHub Codespaces.

### Run the App

```bash
pnpm install
pnpm tauri dev
```

### Database Setup

The app database initializes automatically. For CLI usage, set up the dev database:

```bash
cd src-tauri
cp .env.example .env
sqlx database create
sqlx migrate run
```

After changing SQL queries, regenerate the cache:

```bash
cargo sqlx prepare
```

## CLI

The `potasko` CLI tests backend functionality without the GUI:

```bash
# Build CLI
cargo build --bin potasko

# Task operations
potasko task list --list 1
potasko task add "Buy groceries" --list 1 --due 2026-01-10 --priority 2
potasko task complete 1

# List operations
potasko list list
potasko list add "Work"

# Account operations (CalDAV)
potasko account list
potasko account add --name "Local" --server http://localhost:5232 --username test --password test
potasko account test 1

# Options
potasko --format json task list --list 1   # JSON output
potasko --database ./custom.db task list   # Custom DB path
```

## CalDAV Testing with Radicale

Radicale is a lightweight CalDAV server that auto-starts with the devcontainer.

**Test credentials:** `test` / `test`

### Quick Start

```bash
cd src-tauri

# Add the test account
./target/debug/potasko account add --name "Radicale" --server http://localhost:5232 --username test --password test

# Test connection and discover calendars
./target/debug/potasko account test 1

# Create a calendar
curl -u test:test -X MKCALENDAR "http://localhost:5232/test/tasks/"

# Link a list to the calendar
./target/debug/potasko list link 1 --account 1 --calendar-url "http://localhost:5232/test/tasks/"

# Sync
./target/debug/potasko sync list 1
```

### Reset Test Data

```bash
rm -rf /tmp/radicale-data
```

Or simply restart the devcontainer.

### Manual Radicale Control

```bash
# Check if running
pgrep -a radicale

# Restart
pkill radicale && radicale &
```

## Project Documentation

- `PLAN.md` - Architecture and feature roadmap
- `IMPLEMENTATION.md` - Detailed implementation notes and session log
