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

# Sync operations
potasko sync account 1   # Discover calendars + sync all lists
potasko sync list 1      # Sync a single list

# Options
potasko --format json task list --list 1   # JSON output
potasko --database ./custom.db task list   # Custom DB path
```

## CalDAV Testing with Radicale

Radicale is a lightweight CalDAV server that auto-starts with the devcontainer (auth disabled for development).

### Quick Start

```bash
cd src-tauri

# Add the test account (any username/password works)
./target/debug/potasko account add --name "Radicale" --server http://localhost:5232 --username test --password test

# Create a calendar on the server
curl -u test:test -X MKCALENDAR "http://localhost:5232/test/tasks/"

# Sync account - discovers calendars and imports them as lists
./target/debug/potasko sync account 1

# View imported lists
./target/debug/potasko list list

# Add a task and sync
./target/debug/potasko task add "Test task" --list 2
./target/debug/potasko sync account 1
```

### Backend E2E Tests

Run the sync integration tests against Radicale:

```bash
cd src-tauri

# Ensure Radicale is running
sudo service radicale start

# Run E2E tests (sequential due to shared server)
cargo test --test sync_e2e -- --test-threads=1
```

### UI E2E Tests

Run end-to-end tests for the GUI using WebdriverIO and tauri-driver:

```bash
pnpm test:e2e
```

This will automatically:
- Build the Tauri app in debug mode
- Start Radicale for CalDAV testing
- Launch tauri-driver
- Run tests that interact with the actual GUI

### Reset Test Data

```bash
rm -rf /tmp/radicale-data
```

Or simply restart the devcontainer.

### Manual Radicale Control

```bash
# Check status
sudo service radicale status

# Start/stop/restart
sudo service radicale start
sudo service radicale stop
sudo service radicale restart
```

## Project Documentation

- `PLAN.md` - Architecture and feature roadmap
- `IMPLEMENTATION.md` - Detailed implementation notes and session log
