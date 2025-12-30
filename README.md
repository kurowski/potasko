# Potasko

<p align="center">
  <img src="docs/logo.png" alt="Potasko Logo" width="180">
</p>

<p align="center">
  <strong>Your little task potato, a CalDAV todo app.</strong>
</p>

<p align="center">
  A simple task manager built with Tauri that syncs with CalDAV servers.<br>
  Built for Linux, coming soon to Android.
</p>

## Features

### 📝 Task Management
- ✓ Task completion tracking
- ✓ Due dates and priorities
- ○ Recurring tasks (RRULE) - *planned*
- ✓ Multiple task lists

### 🔄 CalDAV Sync
- ✓ Works with Nextcloud, Radicale, Baikal
- ○ Secure credential storage - *planned*
- ○ Automatic conflict resolution - *planned*
- ○ Background synchronization - *planned*

### 💾 Offline-First
- ✓ Local SQLite storage
- ✓ Work without internet
- ○ Sync when connected - *planned*
- ○ Change queue with retry - *planned*

## Tech Stack

- **Tauri 2.0** - Cross-platform desktop framework
- **Rust** - Backend and core logic
- **SvelteKit** - Modern frontend framework
- **SQLite** - Local database
- **CalDAV** - Calendar and task synchronization protocol

## Development

### Prerequisites
- Rust (latest stable)
- Node.js (v18 or higher)
- pnpm

### Setup
```bash
# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Project Links

- **Website**: [potasko.app](https://potasko.app)
- **GitHub**: [github.com/kurowski/potasko](https://github.com/kurowski/potasko)

---

<p align="center">
  Built with ❤️ in Bratt, VT and Phila, PA
</p>
