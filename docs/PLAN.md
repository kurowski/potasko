# Potasko - CalDAV Task Manager

## Overview
A Tauri 2.0 Linux desktop app for task management with CalDAV sync, designed for future Android compatibility.

**Stack:** Tauri 2.0 (Rust) + SvelteKit + SQLite

## Features
- Basic tasks: title, description, due date, priority, completion
- Recurring tasks (RRULE support)
- Multiple task lists (maps to CalDAV calendars)
- Offline-first with local SQLite storage
- Generic CalDAV sync (Nextcloud, Radicale, Baikal, etc.)

---

## Project Structure

```
potasko/
├── src-tauri/                      # Rust backend
│   ├── src/
│   │   ├── main.rs                 # Tauri GUI entry point
│   │   ├── lib.rs                  # Library exports for CLI + Tauri
│   │   ├── bin/
│   │   │   └── cli.rs              # CLI binary entry point
│   │   ├── core/                   # Pure business logic (shared by CLI + Tauri)
│   │   │   ├── mod.rs
│   │   │   ├── error.rs            # Shared CoreError type
│   │   │   ├── tasks.rs            # Task operations
│   │   │   ├── lists.rs            # List operations
│   │   │   └── accounts.rs         # Account operations
│   │   ├── commands/               # Tauri commands (thin wrappers around core)
│   │   ├── cli/                    # CLI-specific code
│   │   │   ├── mod.rs
│   │   │   ├── args.rs             # clap definitions
│   │   │   └── output.rs           # Table/JSON formatting
│   │   ├── db/                     # SQLite layer (init, migrations)
│   │   ├── caldav/                 # CalDAV client
│   │   │   ├── client.rs           # HTTP client (PROPFIND, PUT, DELETE, etc.)
│   │   │   ├── discovery.rs        # Endpoint discovery
│   │   │   ├── vtodo.rs            # VTODO parsing/building
│   │   │   └── xml.rs              # XML parsing helpers
│   │   ├── sync/                   # Sync engine
│   │   │   ├── engine.rs           # Main sync orchestration
│   │   │   ├── push.rs             # Push local changes to server
│   │   │   ├── pull.rs             # Pull server changes to local
│   │   │   └── types.rs            # SyncResult, SyncStats, etc.
│   │   └── models/                 # Domain models
│   └── tests/                      # E2E tests
│       ├── sync_e2e.rs             # Sync integration tests
│       └── common/mod.rs           # Test helpers
├── src/                            # SvelteKit frontend
│   ├── lib/
│   │   ├── components/             # UI components
│   │   ├── stores/                 # Svelte stores
│   │   │   ├── tasks.svelte.ts     # Task state
│   │   │   ├── lists.svelte.ts     # List state
│   │   │   ├── accounts.svelte.ts  # Account state
│   │   │   └── sync.svelte.ts      # Sync state
│   │   ├── api/                    # Tauri command wrappers
│   │   └── types/                  # TypeScript types
│   └── routes/                     # Pages
├── package.json
└── svelte.config.js
```

---

## Key Dependencies

### Rust (Cargo.toml)
```toml
tauri = "2"
tauri-plugin-sql = { version = "2", features = ["sqlite"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
reqwest = { version = "0.12", features = ["rustls-tls"] }
icalendar = { version = "0.16", features = ["parser"] }
rrule = "0.13"
quick-xml = "0.38"
roxmltree = "0.21"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
thiserror = "2"

# CLI
clap = { version = "4.5", features = ["derive"] }
comfy-table = "7.2"
dirs = "6.0"
```

### Frontend (package.json)
- `@sveltejs/adapter-static` (required for Tauri)
- `@tauri-apps/api`
- `@tauri-apps/plugin-sql`

---

## Database Schema (SQLite)

### Core Tables
1. **accounts** - CalDAV server credentials and discovery URLs
2. **task_lists** - Local lists mapped to CalDAV calendars (with ctag, sync_token)
3. **tasks** - VTODO data with sync metadata (uid, etag, local_version, synced_version)
4. **sync_log** - Audit trail for debugging

### Key Sync Fields on Tasks
- `uid` - iCalendar UID
- `caldav_href` - Resource URL on server
- `caldav_etag` - For conflict detection
- `raw_icalendar` - Preserve unknown properties
- `local_version` / `synced_version` - Change tracking
- `sync_status` - pending | synced | conflict

---

## Sync Strategy

1. **Push local changes first** (create/update/delete with If-Match ETag)
2. **Check CTag** - Skip pull if collection unchanged
3. **Compare ETags** - Identify new/modified/deleted items
4. **Batch download** changes via REPORT multiget
5. **Conflict resolution** - Server wins (fetch and overwrite local)
6. **Background sync** - Every 5 minutes + on app resume

---

## Implementation Phases

### Phase 1: Project Setup & Local Storage ✓
- [x] Initialize Tauri 2.0 + SvelteKit project
- [x] Configure adapter-static for SPA mode
- [x] Set up SQLite with migrations
- [x] Implement Task/TaskList models in Rust
- [x] Create Tauri commands for CRUD
- [x] Build basic UI: list sidebar, task list, task form

### Phase 2: Task Features ✓
- [x] Task completion toggle
- [x] Priority picker (1-9, displayed as High/Medium/Low)
- [x] Due date picker with today/overdue views
- [x] Multiple lists with colors
- [x] Recurrence picker (daily/weekly/monthly/yearly)
- [x] RRULE storage and display

### Phase 3: CalDAV Client ✓
- [x] CalDAV discovery (well-known, principal, calendar-home)
- [x] HTTP Basic authentication
- [x] List calendars with VTODO support
- [x] **CLI for backend testing** (potasko-cli binary)
- [x] Account setup UI with connection test
- [ ] Secure password storage (deferred to Phase 7: Polish)

### Phase 4: VTODO Conversion ✓
- [x] Parse VTODO (icalendar crate): SUMMARY, DESCRIPTION, DUE, PRIORITY, STATUS, RRULE
- [x] Build VTODO with UID generation
- [x] Preserve unknown properties in raw_icalendar
- [x] Round-trip testing

### Phase 5: Basic Sync ✓
- [x] Initial download of all VTODOs
- [x] Store ETags and raw iCalendar
- [x] Implement push (PUT with If-Match)
- [x] Implement pull (CTag check, ETag comparison, multiget)
- [x] Calendar discovery during sync (auto-import new calendars)
- [x] Sync status UI and manual sync button
- [x] E2E tests for sync operations

### Phase 6: Conflict Resolution & Reliability
- [x] Detect 412 Precondition Failed
- [x] Server-wins conflict resolution (fetch and overwrite local)
- [x] Eager sync (single-task push on mutation)
- [x] Offline change queue with retry (error tracking, auto-retry on sync)
- [x] Background sync scheduler (5 min interval + Tauri events)
- [x] Sync logging for debugging

### Phase 7: Polish
- [ ] Keyboard shortcuts
- [ ] Dark mode
- [ ] Error messages and offline indicators
- [ ] Test against Nextcloud, Radicale, Baikal
- [ ] Android compatibility testing

---

## Critical Files

| File | Purpose |
|------|---------|
| `src-tauri/src/core/tasks.rs` | Task business logic (shared by CLI + Tauri) |
| `src-tauri/src/core/lists.rs` | List business logic |
| `src-tauri/src/core/accounts.rs` | Account business logic |
| `src-tauri/src/core/error.rs` | Shared error type |
| `src-tauri/src/models/task.rs` | Task domain model |
| `src-tauri/src/commands/tasks.rs` | Tauri CRUD commands (thin wrappers) |
| `src-tauri/src/bin/cli.rs` | CLI binary entry point |
| `src-tauri/src/cli/args.rs` | CLI argument definitions |
| `src-tauri/src/caldav/client.rs` | CalDAV HTTP client |
| `src-tauri/src/caldav/discovery.rs` | CalDAV endpoint discovery |
| `src-tauri/src/caldav/vtodo.rs` | VTODO parsing and building |
| `src-tauri/src/sync/engine.rs` | Core sync algorithm (push/pull/discovery) |
| `src-tauri/src/sync/push.rs` | Push local changes to server |
| `src-tauri/src/sync/pull.rs` | Pull server changes to local |
| `src/lib/stores/tasks.svelte.ts` | Svelte task store |
| `src/lib/stores/sync.svelte.ts` | Svelte sync state store |
| `src/lib/api/index.ts` | Tauri command wrappers |

---

## Known Challenges

1. **CalDAV server differences** - Test against multiple servers early
2. **VTODO support varies** - Filter by supported-calendar-component-set
3. **Timezone handling** - Store UTC internally, use chrono-tz
4. **Recurring task completion** - Don't complete all instances
5. **Tauri async commands** - Use owned types, not borrowed
6. **Mobile paths** - Use Tauri's path APIs for Android compatibility
7. **Credential storage** - Use tauri-plugin-stronghold or system keyring
8. **Edit during background sync** - If a task is modified on the server while the user is editing it locally, the user's save will silently overwrite the server version (last-write-wins). This is intentional to avoid interrupting the user's workflow.

---

## Future Improvements

1. **tauri-specta** - Generate TypeScript types and typed `invoke()` wrappers from Rust. Eliminates manual type duplication in `src/lib/types/`. Add when type drift becomes a problem or more commands are added.

2. **sync-token Support (RFC 6578)** - For maximum CalDAV server compatibility, implement collection change detection with fallback:
   1. Try sync-token first (RFC 6578) - the IETF-standardized approach
   2. Fall back to CTag (Apple extension) - widely supported but deprecated
   3. Fall back to full ETag comparison - if neither is supported

   This ensures compatibility with servers that only support the newer standard while maintaining backward compatibility. Apple's CTag spec now includes a deprecation notice in favor of RFC 6578.
