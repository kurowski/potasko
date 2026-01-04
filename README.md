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

Radicale is a lightweight CalDAV server included in the devcontainer:

```bash
# Start Radicale (no auth, stores in /tmp)
radicale --storage-filesystem-folder=/tmp/radicale
```

Access at http://localhost:5232. Enter any username/password to create a user, then create a calendar through the web UI.

Test with CLI:

```bash
potasko account add --name "Radicale" --server http://localhost:5232 --username testuser --password testpass
potasko account test 1
```

## Project Documentation

- `PLAN.md` - Architecture and feature roadmap
- `IMPLEMENTATION.md` - Detailed implementation notes and session log
