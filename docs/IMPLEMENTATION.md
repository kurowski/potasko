# Potasko Implementation Plan

## Project Context

- **Discipline**: User wants to review every single step carefully
- **Styling**: Plain CSS (no Tailwind/frameworks)
- **Approach**: Incremental - one step at a time with review between each

---

# Phase 1: Project Setup & Local Storage

## Step 1.1: Project Scaffolding

**Status**: [x] Complete

### Goal

Initialize a Tauri 2.0 project with SvelteKit frontend that compiles and runs.

### Actions

1. **Create Tauri 2.0 + SvelteKit project**

   ```bash
   npm create tauri-app@latest . -- --template sveltekit-ts
   ```

2. **Configure SvelteKit for static adapter**

   - Install `@sveltejs/adapter-static`
   - Update `svelte.config.js`:
     - `fallback: 'index.html'` for SPA routing
     - `prerender: { default: false }`

3. **Update `src/routes/+layout.ts`**

   ```typescript
   export const prerender = false;
   export const ssr = false;
   ```

4. **Verify**: `npm run tauri dev` launches the app

### Files Created

- `package.json`, `svelte.config.js`, `vite.config.ts`
- `src-tauri/Cargo.toml`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- `src/routes/+layout.ts`, `src/routes/+page.svelte`

---

## Step 1.2: SQLite Setup with Migrations

**Status**: [x] Complete

### Goal

Set up SQLite database with sqlx and create the schema for tasks and lists.

### Actions

1. **Add Rust dependencies to `Cargo.toml`**

   ```toml
   sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
   tokio = { version = "1", features = ["full"] }
   ```

2. **Create database module structure**

   ```
   src-tauri/src/
   ├── db/
   │   ├── mod.rs
   │   ├── schema.rs      # Table creation SQL
   │   └── migrations.rs  # Migration runner
   ```

3. **Define schema** (from PLAN.md):

   - `accounts` - CalDAV credentials
   - `task_lists` - Lists with sync metadata
   - `tasks` - VTODO data with sync fields
   - `sync_log` - Audit trail

4. **Initialize DB on app startup**
   - Create DB file in Tauri app data directory
   - Run migrations automatically

### Key Decisions

- Use sqlx compile-time checked queries (requires DATABASE_URL at build)
- Store DB in `tauri::api::path::app_data_dir()`

---

## Step 1.3: Domain Models (Rust)

**Status**: [x] Complete

### Goal

Create Task and TaskList structs with serde serialization.

### Actions

1. **Create models module**

   ```
   src-tauri/src/
   ├── models/
   │   ├── mod.rs
   │   ├── task.rs
   │   └── task_list.rs
   ```

2. **Task struct fields** (from PLAN.md):

   - `id: i64` (SQLite rowid)
   - `uid: String` (iCalendar UID)
   - `list_id: i64`
   - `title: String`
   - `description: Option<String>`
   - `due_date: Option<DateTime<Utc>>`
   - `priority: Option<i32>` (1-9)
   - `completed: bool`
   - `completed_at: Option<DateTime<Utc>>`
   - `rrule: Option<String>`
   - Sync fields: `caldav_href`, `caldav_etag`, `raw_icalendar`, `sync_status`

3. **TaskList struct fields**:
   - `id: i64`
   - `account_id: Option<i64>`
   - `name: String`
   - `color: Option<String>`
   - `caldav_url: Option<String>`
   - `ctag: Option<String>`

### Dependencies

```toml
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
serde = { version = "1", features = ["derive"] }
```

---

## Step 1.4: Tauri Commands for CRUD

**Status**: [x] Complete

### Goal

Expose Task/TaskList operations to the frontend via Tauri commands.

### Actions

1. **Create commands module**

   ```
   src-tauri/src/
   ├── commands/
   │   ├── mod.rs
   │   ├── tasks.rs
   │   └── lists.rs
   ```

2. **Task commands**:

   - `create_task(list_id, title, ...) -> Task`
   - `get_tasks(list_id) -> Vec<Task>`
   - `get_task(id) -> Task`
   - `update_task(id, ...) -> Task`
   - `delete_task(id)`
   - `toggle_task_completion(id) -> Task`

3. **List commands**:

   - `create_list(name, color) -> TaskList`
   - `get_lists() -> Vec<TaskList>`
   - `update_list(id, ...) -> TaskList`
   - `delete_list(id)`

4. **Register commands in `main.rs`**

### Note

Commands use `tauri::State<DbPool>` for database access.

---

## Step 1.5: Basic UI

**Status**: [x] Complete

### Goal

Build minimal functional UI: sidebar with lists, task list view, task form.

### Actions

1. **Create component structure**

   ```
   src/lib/
   ├── components/
   │   ├── Sidebar.svelte       # List navigation
   │   ├── TaskList.svelte      # List of tasks
   │   ├── TaskItem.svelte      # Single task row
   │   └── TaskForm.svelte      # Create/edit task
   ├── stores/
   │   ├── tasks.ts             # Task state
   │   └── lists.ts             # List state
   ├── api/
   │   └── tauri.ts             # Command wrappers
   └── types/
       └── index.ts             # TypeScript types
   ```

2. **Layout**: Two-column (sidebar + main content)

3. **Core interactions**:
   - Select list in sidebar → show tasks
   - Click "+" → show task form
   - Click task → toggle completion
   - Right-click or menu → edit/delete

### Styling

Plain CSS with CSS custom properties for theming.

---

## Progress Tracking

| Step | Description         | Status |
| ---- | ------------------- | ------ |
| 1.1  | Project Scaffolding | [x]    |
| 1.2  | SQLite Setup        | [x]    |
| 1.3  | Domain Models       | [x]    |
| 1.4  | Tauri Commands      | [x]    |
| 1.5  | Basic UI            | [x]    |

---

## Notes for Future Sessions

- Review PLAN.md for full project scope
- Each step should be reviewed before proceeding
- User prefers understanding over speed
- **Package manager**: pnpm

## Session Log

### Session 1 (Step 1.1)

**Completed:**

- Scaffolded Tauri 2.0 + Svelte-TS project using `npx create-tauri-app`
- Template already included: adapter-static, ssr=false, serde
- Installed dependencies with `pnpm install`
- Verified both frontend (`pnpm check`) and backend (`cargo check`) compile
- Added Wayland support to devcontainer.json for GUI testing

**Next steps after container rebuild:**

1. Run `pnpm tauri dev` to verify the app window appears
2. If successful, proceed to Step 1.2: SQLite Setup

### Session 2 (Step 1.1 Verification)

**Completed:**

- Added Svelte extension to devcontainer.json
- Ran `pnpm tauri dev` - full build completed in ~1m 42s
- ✅ App window appeared successfully and was interactive
- Wayland GUI rendering works inside devcontainer (libEGL warnings can be ignored)

**Status: Step 1.1 Complete ✓**

**Ready to proceed to Step 1.2: SQLite Setup**

### Session 3 (Step 1.2)

**Completed:**

- Created `src-tauri/src/db/schema.rs` with table definitions:
  - `accounts` - CalDAV server credentials
  - `task_lists` - Lists with sync metadata (ctag, sync_token)
  - `tasks` - VTODO data with sync fields (uid, etag, local_version, synced_version, sync_status)
  - `sync_log` - Audit trail for debugging
  - Indexes for common query patterns
- Created `src-tauri/src/db/migrations.rs`:
  - Version-based migration system using `schema_version` table
  - `migrate_v1()` creates all tables and a default "Inbox" list
- Updated `src-tauri/src/lib.rs`:
  - Added `db` module
  - Database initialization in Tauri `setup` hook
  - `DbState` wrapper for Tauri state management
  - Database stored in app data directory (`potasko.db`)
- ✅ `cargo check` passes

**Files Created/Modified:**

- `src-tauri/src/db/schema.rs` (new)
- `src-tauri/src/db/migrations.rs` (new)
- `src-tauri/src/lib.rs` (modified)

**Status: Step 1.2 Complete ✓**

**Ready to proceed to Step 1.3: Domain Models**

### Session 4 (Step 1.3)

**Completed:**

- Verified app launches successfully with `pnpm tauri dev`
- Created `src-tauri/src/models/mod.rs` - module exports
- Created `src-tauri/src/models/task.rs`:
  - `Task` struct with all fields matching database schema
  - `SyncStatus` enum (Pending, Synced, Conflict, Deleted)
  - `CreateTask` and `UpdateTask` DTOs for CRUD operations
- Created `src-tauri/src/models/task_list.rs`:
  - `TaskList` struct with all fields matching database schema
  - `CreateTaskList` and `UpdateTaskList` DTOs
- Updated `src-tauri/src/lib.rs` to export models
- ✅ `cargo check` passes

**Files Created:**

- `src-tauri/src/models/mod.rs`
- `src-tauri/src/models/task.rs`
- `src-tauri/src/models/task_list.rs`

**Status: Step 1.3 Complete ✓**

**Ready to proceed to Step 1.4: Tauri Commands**

### Session 5 (Step 1.4)

**Completed:**

- Set up proper sqlx workflow with compile-time checked queries:
  - Created `migrations/20250101000000_initial_schema.sql`
  - Created `.env` and `.env.example` for DATABASE_URL
  - Updated `.gitignore` to ignore `dev.db` but commit `.sqlx/`
  - Replaced custom migration system with `sqlx::migrate!()`
- Created `src-tauri/src/commands/mod.rs` - command exports
- Created `src-tauri/src/commands/lists.rs`:
  - `get_lists()`, `get_list()`, `create_list()`, `update_list()`, `delete_list()`
  - Uses `query_as!` macro with compile-time SQL checking
- Created `src-tauri/src/commands/tasks.rs`:
  - `get_tasks()`, `get_task()`, `create_task()`, `update_task()`, `delete_task()`, `toggle_task_completion()`
  - Uses `as "column!"` syntax to assert non-nullable columns
- Updated `src-tauri/src/lib.rs` to register all commands
- Generated `.sqlx/` cache for offline/CI builds with `cargo sqlx prepare`
- ✅ `cargo check` passes

**Developer Setup (one-time):**

```bash
cd src-tauri
cp .env.example .env
sqlx database create
sqlx migrate run
```

**After changing queries:**

```bash
cargo sqlx prepare
# Commit .sqlx/ changes
```

**Files Created/Modified:**

- `src-tauri/migrations/20250101000000_initial_schema.sql` (new)
- `src-tauri/.env.example` (new)
- `src-tauri/.sqlx/*.json` (new, 12 query cache files)
- `src-tauri/src/commands/mod.rs` (new)
- `src-tauri/src/commands/lists.rs` (new)
- `src-tauri/src/commands/tasks.rs` (new)
- `src-tauri/src/db/mod.rs` (simplified)
- `src-tauri/src/lib.rs` (updated)
- Removed: `src-tauri/src/db/migrations.rs`, `src-tauri/src/db/schema.rs`

**Status: Step 1.4 Complete ✓**

**Ready to proceed to Step 1.5: Basic UI**

### Session 6 (Step 1.5)

**Completed:**

- Created TypeScript types matching Rust models (`src/lib/types/index.ts`)
- Created Tauri API wrapper (`src/lib/api/index.ts`)
- Created Svelte 5 stores using runes:
  - `src/lib/stores/lists.svelte.ts` - list state + CRUD actions
  - `src/lib/stores/tasks.svelte.ts` - task state + CRUD actions
- Created UI components:
  - `Sidebar.svelte` - list navigation with add list form
  - `TaskItem.svelte` - single task with toggle, edit, delete
  - `TaskListView.svelte` - task list with completed section
  - `TaskForm.svelte` - create/edit task form
- Built main page layout (`+page.svelte`) - two-column design
- Added global styles (`app.css`) with CSS variables and dark mode support
- Updated app title to "Potasko"
- ✅ App runs with functional UI

**Files Created:**

- `src/lib/types/index.ts`
- `src/lib/api/index.ts`
- `src/lib/stores/lists.svelte.ts`
- `src/lib/stores/tasks.svelte.ts`
- `src/lib/components/Sidebar.svelte`
- `src/lib/components/TaskItem.svelte`
- `src/lib/components/TaskListView.svelte`
- `src/lib/components/TaskForm.svelte`
- `src/app.css`
- `src/routes/+layout.svelte`
- `src/routes/+page.svelte` (replaced)

**Status: Step 1.5 Complete ✓**

**Phase 1 Complete!** Ready to proceed to Phase 2: Task Features

---

---

# Phase 2: Task Features

## Overview

Enhance the task management experience with better UI for priorities, due dates, list customization, and recurring tasks.

**Note:** Task completion toggle already works from Phase 1.

---

## Step 2.1: Priority Display Enhancement

**Status**: [x] Complete

### Goal

Improve priority picker and display (currently shows "High (1)" etc., could be cleaner).

### Actions

- Update TaskForm priority select with better labels
- Style priority badges consistently
- Consider priority icons or color indicators

### Session 7 (Step 2.1)

**Completed:**

- Added CSS variables for priority colors in `app.css`:
  - Light mode: `--priority-high-*`, `--priority-medium-*`, `--priority-low-*`
  - Dark mode variants for proper contrast
- Updated `TaskItem.svelte` to use CSS variables instead of hardcoded colors
- Cleaned up `TaskForm.svelte`:
  - Simplified labels: "High (1)" → "High", etc.
  - Added colored dot indicator next to the select dropdown
- All changes support dark mode automatically

**Files Modified:**

- `src/app.css` - Added priority color variables
- `src/lib/components/TaskItem.svelte` - Use CSS variables
- `src/lib/components/TaskForm.svelte` - Cleaner labels + color indicator

---

## Step 2.2: Due Date Views

**Status**: [x] Complete

### Goal

Add filtered views for "Today" and "Overdue" tasks across all lists.

### Actions

- Add special list items in sidebar (Today, Overdue)
- Create backend queries for filtered tasks
- Update TaskListView to handle virtual lists

### Session 7 (Step 2.2)

**Completed:**

- Added two new Rust commands: `get_tasks_today` and `get_tasks_overdue`
  - Query tasks by `date(due_date) = date('now')` and `< date('now')`
  - Only show incomplete tasks, sorted by due date then priority
- Updated `lists.svelte.ts` store:
  - Introduced `ViewSelection` type: either `{ type: 'list', id }` or `{ type: 'special', view }`
  - Added `selectedSpecialView` and `selectSpecial()` for special views
  - Fixed TypeScript narrowing issue with union types in ternaries
- Updated `tasks.svelte.ts` store with `loadSpecial()` method
- Updated `Sidebar.svelte`:
  - Added "Views" section with Today/Overdue buttons
  - Shows task list below with "Lists" header
- Updated `TaskListView.svelte`:
  - Dynamic title based on view type
  - Hide "Completed" section in special views (they only show incomplete)
- Updated `+page.svelte`:
  - Handle both list and special views in $effect
  - Hide TaskForm in special views (can't add tasks to virtual lists)
- Generated new sqlx query cache

**Files Modified:**

- `src-tauri/src/commands/tasks.rs` - New commands
- `src-tauri/src/lib.rs` - Register commands
- `src-tauri/.sqlx/` - Query cache
- `src/lib/api/index.ts` - API wrappers
- `src/lib/stores/lists.svelte.ts` - View selection logic
- `src/lib/stores/tasks.svelte.ts` - loadSpecial method
- `src/lib/components/Sidebar.svelte` - Special view buttons
- `src/lib/components/TaskListView.svelte` - Dynamic titles
- `src/routes/+page.svelte` - View switching logic

---

## Step 2.3: List Colors

**Status**: [x] Complete

### Goal

Allow users to pick colors for lists (already stored in DB, need UI).

### Actions

- Add color picker to list creation/editing
- Display color in sidebar
- Consider predefined color palette

### Session 7 (Step 2.3)

**Completed:**

- Added predefined 9-color palette (gray, red, orange, yellow, green, teal, blue, purple, pink)
- Updated "Add list" form with color picker row
- Added edit functionality:
  - Click on color square to edit list (name + color)
  - Inline edit form replaces the list item
  - Escape key cancels editing
- Refactored list item to use separate buttons for color and name
- Added CSS for color picker (selected state with border)

**Files Modified:**

- `src/lib/components/Sidebar.svelte` - Color picker UI and edit functionality

---

## Step 2.4: Recurrence (RRULE)

**Status**: [x] Complete

### Goal

Allow users to create recurring tasks with basic patterns.

### Actions

- Add recurrence picker to TaskForm (daily/weekly/monthly/yearly)
- Generate RRULE string from selection
- Display recurrence info on tasks
- (Full RRULE parsing deferred to Phase 4 with CalDAV)

### Session 7 (Step 2.4)

**Completed:**

- Added recurrence picker to TaskForm:
  - Options: None, Daily, Weekly, Monthly, Yearly
  - Stores standard RRULE format (e.g., `FREQ=DAILY`)
- Updated TaskItem to display recurrence:
  - Shows repeat icon + label (e.g., "Daily")
  - Parses RRULE to extract frequency
- Form state properly resets rrule on submit
- **Backend recurrence logic** (`toggle_task_completion`):
  - When completing a recurring task with a due date, creates the next occurrence
  - `calculate_next_due_date()` handles Daily, Weekly, Monthly, Yearly frequencies
  - Monthly handles edge cases (e.g., Jan 31 → Feb 28)
- **Frontend reload** after completing recurring tasks:
  - `taskStore` tracks current list/view for reload capability
  - `toggle()` detects recurring task completion and triggers reload

**Files Modified:**

- `src/lib/components/TaskForm.svelte` - Recurrence picker
- `src/lib/components/TaskItem.svelte` - Recurrence display
- `src-tauri/src/commands/tasks.rs` - Recurrence logic in toggle + date calculation
- `src/lib/stores/tasks.svelte.ts` - Track current view, reload on recurrence

---

## Progress Tracking

| Step | Description      | Status |
| ---- | ---------------- | ------ |
| 2.1  | Priority Display | [x]    |
| 2.2  | Due Date Views   | [x]    |
| 2.3  | List Colors      | [x]    |
| 2.4  | Recurrence       | [x]    |

---

**Phase 2 Complete!** Ready to proceed to Phase 3: CalDAV Client

---

---

# Phase 3: CalDAV Client

## Overview

Add CalDAV client capabilities to Potasko: account management, server discovery, and calendar listing with VTODO support.

**Password Storage**: Plaintext for development. TODO: Implement proper credential storage before release using `tauri-plugin-stronghold` or system keyring.

---

## Pre-requisite: Radicale Setup

### Option A: System Install (Recommended for devcontainer)

```bash
# Install
pip install radicale

# Create config
mkdir -p ~/.config/radicale
cat > ~/.config/radicale/config << 'EOF'
[server]
hosts = 0.0.0.0:5232

[auth]
type = htpasswd
htpasswd_filename = ~/.config/radicale/users
htpasswd_encryption = plain

[storage]
filesystem_folder = ~/.var/lib/radicale/collections
EOF

# Create test user
echo "testuser:testpass" > ~/.config/radicale/users

# Create storage directory
mkdir -p ~/.var/lib/radicale/collections

# Run
radicale --config ~/.config/radicale/config
```

### Option B: Minimal (No auth)

```bash
pip install radicale
radicale --storage-filesystem-folder=/tmp/radicale
# Access at http://localhost:5232 with any username/password
```

### Test Endpoints

- Base URL: `http://localhost:5232`
- Well-known: `http://localhost:5232/.well-known/caldav`
- User principal: `http://localhost:5232/testuser/`

---

## Step 3.1: Add Dependencies

**Status**: [x] Complete

### Cargo.toml additions

```toml
# HTTP client
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls"] }

# XML parsing for CalDAV
quick-xml = "0.37"
roxmltree = "0.20"
```

### Files Modified

- `src-tauri/Cargo.toml`

---

## Step 3.2: Account Model

**Status**: [x] Complete

### Goal

Create Account struct matching existing database schema.

### New file: `src-tauri/src/models/account.rs`

```rust
pub struct Account {
    pub id: i64,
    pub name: String,
    pub server_url: String,
    pub username: String,
    pub password: String,
    pub principal_url: Option<String>,
    pub calendar_home_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CreateAccount { ... }
pub struct UpdateAccount { ... }
```

### Files Modified

- `src-tauri/src/models/mod.rs`
- `src-tauri/src/models/account.rs` (new)

---

## Step 3.3: CalDAV Client Module

**Status**: [x] Complete

### Goal

Core HTTP client for CalDAV operations.

### Directory structure

```
src-tauri/src/caldav/
├── mod.rs         # Module exports
├── client.rs      # HTTP client wrapper with auth
├── discovery.rs   # Well-known, principal, calendar-home discovery
├── xml.rs         # DAV XML request/response parsing
└── types.rs       # CalDAV-specific types (Calendar, etc.)
```

### Key types

```rust
pub struct CalendarInfo {
    pub href: String,
    pub display_name: Option<String>,
    pub color: Option<String>,
    pub ctag: Option<String>,
    pub supports_vtodo: bool,
}

pub struct CalDavClient {
    http: reqwest::Client,
    base_url: String,
    username: String,
    password: String,
}
```

### Discovery Flow

1. Try `/.well-known/caldav` → follow redirect to principal
2. If no well-known, PROPFIND on `/` for `current-user-principal`
3. PROPFIND on principal for `calendar-home-set`
4. PROPFIND on calendar-home for calendars with `supported-calendar-component-set` containing VTODO

### Files Created

- `src-tauri/src/caldav/mod.rs`
- `src-tauri/src/caldav/client.rs`
- `src-tauri/src/caldav/discovery.rs`
- `src-tauri/src/caldav/xml.rs`
- `src-tauri/src/caldav/types.rs`

---

## Step 3.4: Account Commands

**Status**: [x] Complete

### Goal

Tauri commands for account CRUD and connection testing.

### Commands

```rust
get_accounts() -> Vec<Account>
create_account(data: CreateAccount) -> Account
update_account(id, data: UpdateAccount) -> Account
delete_account(id)
test_account_connection(server_url, username, password) -> AccountTestResult
```

### AccountTestResult

```rust
pub struct AccountTestResult {
    pub success: bool,
    pub principal_url: Option<String>,
    pub calendar_home_url: Option<String>,
    pub calendars: Vec<CalendarInfo>,
    pub error: Option<String>,
}
```

### Files Modified

- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/commands/accounts.rs` (new)
- `src-tauri/src/lib.rs`

---

## Step 3.5: Account Setup UI

**Status**: [x] Complete

### Goal

UI for adding/managing CalDAV accounts.

### New Components

```
src/lib/components/
├── AccountForm.svelte      # Add/edit account form
├── AccountList.svelte      # List of configured accounts
└── CalendarPicker.svelte   # Select calendars to sync
```

### AccountForm.svelte

- Fields: Name, Server URL, Username, Password
- "Test Connection" button → shows discovered calendars
- "Save" button → saves account + selected calendars as task lists

### Integration

- Add "Settings" button in Sidebar
- Opens settings panel with AccountList

### Files Created

- `src/lib/components/AccountForm.svelte`
- `src/lib/components/AccountList.svelte`
- `src/lib/components/CalendarPicker.svelte`
- `src/lib/stores/accounts.svelte.ts`

### Files Modified

- `src/lib/types/index.ts`
- `src/lib/api/index.ts`
- `src/lib/components/Sidebar.svelte`
- `src/routes/+page.svelte`

---

## Progress Tracking

| Step | Description          | Status |
| ---- | -------------------- | ------ |
| 3.1  | Add Dependencies     | [x]    |
| 3.2  | Account Model        | [x]    |
| 3.3  | CalDAV Client Module | [x]    |
| 3.4  | Account Commands     | [x]    |
| 3.5  | Account Setup UI     | [x]    |

---

### Session 9 (Steps 3.2-3.4)

**Completed:**

- Finished Step 3.2: Added Account exports to `lib.rs`
- Step 3.3: Created CalDAV client module:
  - `caldav/types.rs` - CalendarInfo, AccountTestResult types
  - `caldav/client.rs` - HTTP client with PROPFIND, auth, error handling
  - `caldav/xml.rs` - DAV XML parsing with roxmltree
  - `caldav/discovery.rs` - Well-known, principal, calendar-home discovery
- Step 3.4: Created account commands:
  - `get_accounts`, `get_account`, `create_account`, `update_account`, `delete_account`
  - `test_account_connection` - discovers CalDAV endpoints
- Created migrations:
  - `20250102000000_rename_password_column.sql` - renamed `password_encrypted` to `password`
  - `20250103000000_password_not_null.sql` - made password NOT NULL
- Added sqlx-cli to devcontainer Dockerfile

**Files Created:**

- `src-tauri/src/caldav/mod.rs`
- `src-tauri/src/caldav/types.rs`
- `src-tauri/src/caldav/client.rs`
- `src-tauri/src/caldav/xml.rs`
- `src-tauri/src/caldav/discovery.rs`
- `src-tauri/src/commands/accounts.rs`
- `src-tauri/migrations/20250102000000_rename_password_column.sql`
- `src-tauri/migrations/20250103000000_password_not_null.sql`

**Files Modified:**

- `src-tauri/src/lib.rs` - added caldav module, account exports, commands
- `src-tauri/src/commands/mod.rs` - added accounts module
- `.devcontainer/Dockerfile` - added sqlx-cli install

**Status: Steps 3.1-3.4 Complete ✓**

**Ready to proceed to Step 3.5: Account Setup UI**

---

### Session 10 (CLI Implementation)

**Status:** Complete ✓

**Goal:** Add CLI binary for testing backend without GUI

**Architecture:** Thin Wrapper Pattern

- Extracted pure business logic into `core/` module
- Both Tauri commands and CLI call `core::*` functions
- Core functions take `&SqlitePool` directly (no Tauri state)

**Completed:**

- Step 1: Added CLI dependencies to Cargo.toml:
  - `clap = "4.5"` with derive feature
  - `comfy-table = "7.2"` for table output
  - `dirs = "6.0"` for default paths
  - Added `[[bin]]` target for `potasko-cli`
- Step 2: Created `core/` module with pure business logic:
  - `core/error.rs` - shared CoreError type
  - `core/tasks.rs` - all task functions taking `&SqlitePool`
  - `core/lists.rs` - all list functions
  - `core/accounts.rs` - all account functions
- Step 3: Refactored Tauri commands to thin wrappers calling `core::*`
- Step 4: Created CLI infrastructure:
  - `cli/mod.rs` - module exports + `default_db_path()`
  - `cli/args.rs` - clap argument definitions
  - `cli/output.rs` - table/JSON formatting with comfy-table
- Step 5: Implemented full CLI binary (`src/bin/cli.rs`):
  - Task commands: list, add, complete, delete, show
  - List commands: list, add, delete
  - Account commands: list, add (with connection test), test, delete
  - Global flags: --database, --format (table/json)
  - Date parsing, recurrence shortcuts (daily/weekly/monthly/yearly)
- Step 6: Regenerated sqlx cache and tested all commands

**Files Created:**

- `src-tauri/src/core/mod.rs`
- `src-tauri/src/core/error.rs`
- `src-tauri/src/core/tasks.rs`
- `src-tauri/src/core/lists.rs`
- `src-tauri/src/core/accounts.rs`
- `src-tauri/src/cli/mod.rs`
- `src-tauri/src/cli/args.rs`
- `src-tauri/src/cli/output.rs`
- `src-tauri/src/bin/cli.rs`

**Files Modified:**

- `src-tauri/Cargo.toml` - added deps and bin target
- `src-tauri/src/lib.rs` - exported core, db, cli modules
- `src-tauri/src/commands/tasks.rs` - thin wrapper
- `src-tauri/src/commands/lists.rs` - thin wrapper
- `src-tauri/src/commands/accounts.rs` - thin wrapper

**CLI Usage Examples:**

```bash
# List all lists
potasko-cli list list

# Add a task
potasko-cli task add "Buy groceries" --list 1 --due 2026-01-05 --priority 2

# List tasks for a list
potasko-cli task list --list 1

# List today's tasks
potasko-cli task list --today

# Complete a task
potasko-cli task complete 1

# JSON output for scripting
potasko-cli --format json task list --list 1

# Add recurring task
potasko-cli task add "Weekly review" --list 1 --due 2026-01-05 --recur weekly

# Account management
potasko-cli account list
potasko-cli account add --name "Work" --server https://caldav.example.com --username me --password secret
potasko-cli account test 1
```

**Status: CLI Implementation Complete ✓**

**Ready to proceed to Step 3.5: Account Setup UI**

---

### Session 11 (Step 3.5)

**Status:** Complete ✓

**Goal:** Add Account Setup UI for managing CalDAV accounts

**Completed:**

- Added TypeScript types for Account, CreateAccount, UpdateAccount, CalendarInfo, AccountTestResult
- Added API functions: getAccounts, getAccount, createAccount, updateAccount, deleteAccount, testAccountConnection
- Created `accounts.svelte.ts` store with reactive state for accounts, loading, testing, testResult
- Created `AccountForm.svelte`:
  - Form fields: Account Name, Server URL, Username, Password
  - "Test Connection" button with discovered calendars display
  - Saves account with discovered principal/calendar-home URLs
- Created `AccountList.svelte`:
  - Lists configured accounts with edit/delete actions
  - Empty state when no accounts configured
  - Add Account button
- Updated `Sidebar.svelte`:
  - Added Settings button at bottom of sidebar
  - Settings button toggles settings panel visibility
- Updated `+page.svelte`:
  - Added showSettings state
  - Toggles between task view and settings panel

**Files Created:**

- `src/lib/stores/accounts.svelte.ts`
- `src/lib/components/AccountForm.svelte`
- `src/lib/components/AccountList.svelte`

**Files Modified:**

- `src/lib/types/index.ts` - Added Account and CalDAV types
- `src/lib/api/index.ts` - Added Account API functions
- `src/lib/components/Sidebar.svelte` - Added Settings button
- `src/routes/+page.svelte` - Added settings panel toggle

**Status: Step 3.5 Complete ✓**

**Phase 3 Complete!** Ready to proceed to Phase 4: VTODO Conversion

---

---

# Phase 4: VTODO Conversion

## Overview

Implement conversion between local Task model and iCalendar VTODO format for CalDAV sync.

---

## Step 4.1: VTODO Module Implementation

**Status**: [x] Complete

### Goal

Create a module to parse and build VTODO components from iCalendar data.

### Session 12 (Step 4.1)

**Completed:**

- Added dependencies to Cargo.toml:
  - `icalendar = "0.17"` with parser feature
  - `rrule = "0.14"`
- Created `src-tauri/src/caldav/vtodo.rs` module:
  - `ParsedVTodo` struct for extracted VTODO data
  - `VTodoBuildData` struct for building VTODOs
  - `VTodoError` enum for error handling
  - `parse_vtodo()` - parses iCalendar string, extracts first VTODO
  - `build_vtodo()` - builds iCalendar string from task data
  - `preserve_unknown_properties()` - copies unhandled properties for round-trip fidelity
- Property mapping implemented:
  - UID ↔ uid
  - SUMMARY ↔ title
  - DESCRIPTION ↔ description
  - DUE ↔ due_date
  - PRIORITY ↔ priority (1-9)
  - STATUS ↔ completed (COMPLETED vs NEEDS-ACTION)
  - COMPLETED ↔ completed_at
  - RRULE ↔ rrule
- Datetime parsing handles:
  - UTC times (20250105T120000Z)
  - Local times (20250105T120000)
  - Date-only values (20250105)
- All 7 unit tests pass:
  - test_parse_vtodo
  - test_parse_completed_vtodo
  - test_build_vtodo
  - test_build_completed_vtodo
  - test_vtodo_with_rrule
  - test_no_vtodo_error
  - test_round_trip_preserves_unknown_properties

**Files Created:**

- `src-tauri/src/caldav/vtodo.rs`

**Files Modified:**

- `src-tauri/Cargo.toml` - Added icalendar and rrule deps
- `src-tauri/src/caldav/mod.rs` - Exported vtodo module

**Status: Step 4.1 Complete ✓**

**Phase 4 Complete!** Ready to proceed to Phase 5: Basic Sync

---

---

# Phase 5: Basic Sync

## Overview

Complete the sync implementation: calendar import, sync UI, and E2E test updates.

**Note:** Backend sync engine (push/pull/CTag/ETag) already implemented. See `src-tauri/src/sync/`.

---

## Step 5.1: Calendar discovery during sync

**Status**: [x] Complete

### Goal

When syncing an account, discover calendars on the server and import any new ones as task lists. This handles both initial setup and calendars created on the server later.

### Flow

1. `account add` - saves credentials (connection test validates them)
2. `sync account <id>` - discovers calendars, imports new ones as lists, syncs tasks

### Actions

1. Add `create_list_from_calendar()` to `core/lists.rs` ✓
2. Add calendar discovery to `SyncEngine::sync_account()` ✓
3. Update frontend to refresh lists after sync (deferred to Step 5.2)
4. Regenerate sqlx cache ✓

### Design Decision

Calendar import is part of normal sync, not a special case during account creation. This ensures the same code path handles initial setup and new server-side calendars.

### Session 14 (Step 5.1 - complete)

**Completed:**

- Made `CalDavClient::discover()` public
- Added `AccountSyncResult` type to `sync/types.rs`
- Implemented `SyncEngine::sync_account()`:
  - Discovers calendars via CalDAV
  - Imports new VTODO-capable calendars as task lists (checks by caldav_url)
  - Syncs all lists for the account
- Updated CLI `sync account` command to use the new method
- Regenerated sqlx cache
- Tested with Radicale: calendar discovery and import works correctly

**Files Modified:**

- `src-tauri/src/caldav/client.rs` - made discover() public
- `src-tauri/src/sync/types.rs` - added AccountSyncResult
- `src-tauri/src/sync/mod.rs` - exported AccountSyncResult
- `src-tauri/src/sync/engine.rs` - added sync_account() method
- `src-tauri/src/bin/cli.rs` - updated sync account handler

---

## Step 5.2: Frontend Sync UI

**Status**: [x] Complete

### Design Decisions (per user)

- Sync button per account (in Settings panel)
- Spinning indicator when syncing
- Sync results shown after completion
- Sidebar shows sync-in-progress indicator

### Session 15 (Step 5.2 - complete)

**Completed:**

- Updated Tauri `sync_account` command to use `SyncEngine::sync_account()` (includes calendar discovery)
- Added `AccountSyncResult` type to frontend types
- Created `src/lib/stores/sync.svelte.ts`:
  - Tracks syncing state (which account, in progress)
  - Stores last result for display
  - Calls list refresh after sync
- Updated `AccountList.svelte`:
  - Added sync button per account (spinning icon when syncing)
  - Shows sync result (success/error, calendars imported, items synced)
- Updated `Sidebar.svelte`:
  - Added sync-in-progress indicator at bottom
- Regenerated sqlx cache

**Files Created:**

- `src/lib/stores/sync.svelte.ts`

**Files Modified:**

- `src-tauri/src/commands/sync.rs` - use SyncEngine::sync_account()
- `src/lib/types/index.ts` - added AccountSyncResult
- `src/lib/api/index.ts` - updated syncAccount return type
- `src/lib/components/AccountList.svelte` - sync button + results
- `src/lib/components/Sidebar.svelte` - sync indicator

---

## Step 5.3: Update E2E tests

**Status**: [x] Complete

Updated E2E tests to use the new sync-based calendar discovery flow:

- Replaced `link_list()` with `sync_account()` helper
- `sync_account()` discovers calendars, imports them, and returns the list ID
- Updated `get_synced_list_id()` to match the specific test calendar URL
- All 8 tests pass

---

## Step 5.4: Update README

**Status**: [x] Complete

Update README to reflect the new workflow:

- Removed `potasko list link` command
- Added `sync account` and `sync list` to CLI examples
- Updated Quick Start to use new workflow: account add → create calendar → sync account

---

## Progress Tracking

| Step | Description                    | Status       |
| ---- | ------------------------------ | ------------ |
| 5.1  | Calendar discovery during sync | [x] Complete |
| 5.2  | Frontend Sync UI               | [x] Complete |
| 5.3  | Update E2E tests               | [x] Complete |
| 5.4  | Update README                  | [x] Complete |

---

## Deferred to Phase 6

- MKCALENDAR (creating calendars on server)
- Account selector when creating local lists

---

---

# Previous Phases - Progress Tracking

## Phase 4 Progress

| Step | Description                 | Status |
| ---- | --------------------------- | ------ |
| 4.1  | VTODO Module Implementation | [x]    |

---

## Test Plan

### Manual Testing with Radicale

1. Start Radicale (`radicale --storage-filesystem-folder=/tmp/radicale`)
2. Add account in UI: `http://localhost:5232`, any user/pass
3. Verify connection test discovers principal + calendars
4. Create a calendar in Radicale web UI
5. Verify it appears in calendar picker
6. Save account, verify task list created

### Debug Commands

```bash
# Test well-known redirect
curl -v http://localhost:5232/.well-known/caldav

# PROPFIND for current-user-principal
curl -X PROPFIND -u testuser:testpass \
  -H "Content-Type: application/xml" \
  -H "Depth: 0" \
  -d '<?xml version="1.0"?><d:propfind xmlns:d="DAV:"><d:prop><d:current-user-principal/></d:prop></d:propfind>' \
  http://localhost:5232/
```

---

---

# Phase 6: Conflict Resolution & Reliability

## Step 6.1: Eager Sync (Single-Task Push)

**Status**: [x] Complete

### Goal

Automatically push changes to the CalDAV server when tasks are created, updated, or deleted.

### Design Decisions

- **Single-task push** instead of full list sync (efficient)
- **Non-blocking** background sync (offline-first UX)
- **Local-only lists** are silently skipped
- **Error handling**: sync errors are logged but don't fail the mutation

### Session 16 (Step 6.1)

**Completed:**

- Added `is_synced_list(list_id, pool)` to `core/lists.rs`
- Added `PushTaskResult` type to `sync/types.rs`
- Added `push_task(task_id)` method to `SyncEngine`
- Made push functions public in `sync/push.rs` (`push_create`, `push_update`, `push_delete`)
- Added `trigger_background_push(task_id, list_id, pool)` helper to `commands/tasks.rs`
- Updated task commands to trigger push after mutations:
  - `create_task` → triggers push
  - `update_task` → triggers push
  - `toggle_task_completion` → triggers push
  - `delete_task` → triggers push only if task was synced (soft-delete)
- Regenerated sqlx cache
- All E2E tests pass

**Files Created/Modified:**

- `src-tauri/src/core/lists.rs` - added `is_synced_list()`
- `src-tauri/src/sync/types.rs` - added `PushTaskResult`
- `src-tauri/src/sync/engine.rs` - added `push_task()` method
- `src-tauri/src/sync/push.rs` - made push functions public, added `PushAction::as_str()`
- `src-tauri/src/sync/mod.rs` - exported `PushTaskResult`
- `src-tauri/src/commands/tasks.rs` - added `trigger_background_push()`, updated all mutation commands

**How It Works:**

1. User creates/updates/deletes a task (via UI)
2. Tauri command executes the core operation
3. After success, spawns a background task to push the change
4. Background task checks if list has CalDAV URL
5. If synced, calls `SyncEngine::push_task()` to push just that one task
6. Push errors are logged but don't affect the user

**Testing:**
Run `pnpm tauri dev`, add a task to a synced list, check Radicale for the new task.

---

## Progress Tracking (Phase 6)

| Step | Description                     | Status       |
| ---- | ------------------------------- | ------------ |
| 6.1  | Eager Sync (Single-Task Push)   | [x] Complete |
| 6.2  | Offline change queue with retry | [ ] Pending  |
| 6.3  | Background sync scheduler       | [ ] Pending  |
| 6.4  | Sync logging for debugging      | [ ] Pending  |

---

---

# Distribution Setup

## Flatpak Configuration

**Status**: [x] Complete

### Session 17 (Flatpak Setup)

**Goal:** Configure Flatpak build for Flathub distribution

**Completed:**

- Fixed `pnpm tauri build` error: missing `xdg-utils` (added to Dockerfile)
- Fixed AppImage bundling wrong binary: renamed Cargo package from `potasko` to `potasko-gui`
- Changed app identifier from `com.vscode.potasko` to `net.kurowski.potasko` everywhere
- Created Flatpak manifest and metadata files:
  - `net.kurowski.potasko.yml` - Flatpak manifest (GNOME 48 runtime)
  - `net.kurowski.potasko.desktop` - Desktop entry
  - `net.kurowski.potasko.metainfo.xml` - AppStream metadata for Flathub
- Added Flatpak tooling to Dockerfile:
  - `flatpak`, `flatpak-builder`
  - `python3-pip`, `python3-aiohttp`, `python3-toml`
  - `tomlkit`, `flatpak-node-generator` (pip)
- Added `--privileged` to devcontainer for flatpak-builder support
- Created `.github/workflows/flatpak.yml` for CI builds

**Files Created:**

- `net.kurowski.potasko.yml`
- `net.kurowski.potasko.desktop`
- `net.kurowski.potasko.metainfo.xml`
- `.github/workflows/flatpak.yml`

**Files Modified:**

- `.devcontainer/Dockerfile` - Added flatpak, python tools
- `.devcontainer/devcontainer.json` - Added `--privileged`
- `.gitignore` - Added Flatpak build artifacts
- `src-tauri/tauri.conf.json` - Changed identifier
- `src-tauri/Cargo.toml` - Renamed package to `potasko-gui`
- `src-tauri/src/cli/mod.rs` - Changed APP_IDENTIFIER
- `src-tauri/src/cli/args.rs` - Updated help text
- Various test files - Updated app identifier paths

---

### Session 18 (Flatpak Build Working)

**Goal:** Get Flatpak build working end-to-end

**Completed:**

- Installed Flatpak runtimes and SDK extensions (GNOME 48, rust-stable, node22)
- Fixed blank window issue: must use `tauri build` instead of separate `npm build` + `cargo build`
- Tauri CLI properly embeds frontend assets during compilation
- Changed `tauri.conf.json` to use `npm run` commands (pnpm not available in Flatpak SDK)
- Added WebKitGTK permissions for dconf access
- All builds now use online dependencies (simpler than vendored sources)

**Key Fix:**

The blank window was caused by using `cargo build` directly, which doesn't embed the frontend assets properly. Using `npm run tauri build` runs the Tauri CLI which handles frontend embedding correctly.

**Build Instructions:**

1. Set up Flatpak runtimes (one-time):
   ```bash
   flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
   flatpak install --user flathub org.gnome.Platform//48 org.gnome.Sdk//48
   flatpak install --user flathub org.freedesktop.Sdk.Extension.rust-stable//24.08
   flatpak install --user flathub org.freedesktop.Sdk.Extension.node22//24.08
   ```

2. Build Flatpak:
   ```bash
   flatpak-builder --user --force-clean build-dir net.kurowski.potasko.yml
   ```

3. Export bundle (for distribution):
   ```bash
   flatpak build-export repo build-dir
   flatpak build-bundle repo potasko.flatpak net.kurowski.potasko
   ```

4. Install and run:
   ```bash
   flatpak install --user potasko.flatpak
   flatpak run net.kurowski.potasko
   ```

**CI/CD:**

The `.github/workflows/flatpak.yml` workflow automatically builds on push/PR and uploads artifacts.
