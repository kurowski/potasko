# Potasko

<p align="center">
  <img src="docs/logo.png" alt="Potasko Logo" width="180">
</p>

<p align="center">
  <strong>Your little task potato, a CalDAV todo app.</strong>
</p>

## Overview

An offline-first CalDAV task manager built with Tauri 2.0, Rust, and SvelteKit. Designed for Linux with planned Android support.

## Architecture

**Frontend**: SvelteKit provides the UI layer with TypeScript
**Backend**: Rust handles business logic, CalDAV protocol, and database operations
**Storage**: SQLite for local task persistence
**Sync**: CalDAV protocol implementation for server synchronization

## Tech Stack

- **Tauri 2.0** - Cross-platform framework leveraging system WebView
- **Rust** - Backend core, database layer, CalDAV client
- **SvelteKit** - Frontend framework with TypeScript
- **SQLite** - Embedded database via rusqlite
- **CalDAV** - RFC 4791 implementation for task sync

## Getting Started

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (v18+) and pnpm
npm install -g pnpm
```

### Development Setup

```bash
# Clone the repository
git clone https://github.com/kurowski/potasko.git
cd potasko

# Install frontend dependencies
pnpm install

# Run development build with hot reload
pnpm tauri dev

# Run tests
cargo test
pnpm test

# Build production binary
pnpm tauri build
```

### Database Migrations

SQLite schema is managed in Rust. Database is initialized on first run.

```bash
# Database location (Linux)
~/.local/share/potasko/tasks.db
```

## Project Structure

```
potasko/
├── src/                    # SvelteKit frontend
│   ├── routes/            # Page components
│   └── lib/               # Shared components and utilities
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs       # Application entry point
│   │   ├── db/           # SQLite database layer
│   │   ├── caldav/       # CalDAV client implementation
│   │   └── commands/     # Tauri commands (IPC)
│   └── Cargo.toml
├── static/                # Static assets
└── docs/                  # GitHub Pages site
```

## Implementation Status

### Implemented

- Local task storage (SQLite)
- Basic task CRUD operations
- Multiple task lists
- Task completion tracking
- Due dates and priorities

### In Progress

- CalDAV client (RFC 4791)
- Server discovery and authentication
- iCalendar (RFC 5545) parsing for VTODO

### Planned

- Recurring tasks (RRULE support)
- Conflict resolution for offline changes
- Background sync service
- Credential storage (keyring integration)
- Android build target

## Development

### Tauri Commands

Rust functions exposed to frontend via `#[tauri::command]`:

```rust
// Example command structure
#[tauri::command]
async fn get_tasks(list_id: String) -> Result<Vec<Task>, String>
```

### Frontend IPC

```typescript
// Invoke Rust commands from SvelteKit
import { invoke } from "@tauri-apps/api/core";
const tasks = await invoke("get_tasks", { listId: "default" });
```

### CalDAV Testing

Use test servers for development:

- **Radicale** - Lightweight CalDAV server
- **Baikal** - Self-hosted alternative
- **Nextcloud** - Full-featured groupware

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + Extensions:

- [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Resources

- **Website**: [potasko.app](https://potasko.app)
- **CalDAV Spec**: [RFC 4791](https://www.rfc-editor.org/rfc/rfc4791)
- **iCalendar Spec**: [RFC 5545](https://www.rfc-editor.org/rfc/rfc5545)
- **Tauri Docs**: [tauri.app](https://tauri.app)
