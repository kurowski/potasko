// Types matching Rust models in src-tauri/src/models/

export type SyncStatus = 'pending' | 'synced' | 'conflict' | 'deleted';

export interface Task {
  id: number;
  list_id: number;
  uid: string;
  title: string;
  description: string | null;
  due_date: string | null;  // ISO 8601 string
  priority: number | null;  // 1-9, 1 = highest
  completed: boolean;
  completed_at: string | null;
  rrule: string | null;
  caldav_href: string | null;
  caldav_etag: string | null;
  raw_icalendar: string | null;
  local_version: number;
  synced_version: number;
  sync_status: SyncStatus;
  created_at: string;
  updated_at: string;
}

export interface CreateTask {
  list_id: number;
  title: string;
  description?: string | null;
  due_date?: string | null;
  priority?: number | null;
  rrule?: string | null;
}

export interface UpdateTask {
  title?: string;
  description?: string | null;
  due_date?: string | null;
  priority?: number | null;
  rrule?: string | null;
}

export interface TaskList {
  id: number;
  account_id: number | null;
  name: string;
  color: string | null;
  caldav_url: string | null;
  ctag: string | null;
  sync_token: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateTaskList {
  name: string;
  color?: string | null;
}

export interface UpdateTaskList {
  name?: string;
  color?: string | null;
}

// --- Account types ---

export interface Account {
  id: number;
  name: string;
  server_url: string;
  username: string;
  password: string;
  principal_url: string | null;
  calendar_home_url: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateAccount {
  name: string;
  server_url: string;
  username: string;
  password: string;
  principal_url?: string | null;
  calendar_home_url?: string | null;
}

export interface UpdateAccount {
  name?: string;
  server_url?: string;
  username?: string;
  password?: string;
  principal_url?: string | null;
  calendar_home_url?: string | null;
}

// --- CalDAV types ---

export interface CalendarInfo {
  href: string;
  display_name: string | null;
  color: string | null;
  ctag: string | null;
  supports_vtodo: boolean;
}

export interface AccountTestResult {
  success: boolean;
  principal_url: string | null;
  calendar_home_url: string | null;
  calendars: CalendarInfo[];
  error: string | null;
}

// --- Sync types ---

export interface SyncStats {
  pushed_created: number;
  pushed_updated: number;
  pushed_deleted: number;
  pulled_created: number;
  pulled_updated: number;
  pulled_deleted: number;
  conflicts: number;
  errors: string[];
}

export interface SyncResult {
  success: boolean;
  list_id: number;
  stats: SyncStats;
  error: string | null;
}

export interface AccountSyncResult {
  success: boolean;
  account_id: number;
  calendars_imported: number;
  list_results: SyncResult[];
  error: string | null;
}

export interface ListSyncStatus {
  list_id: number;
  has_caldav: boolean;
  pending_changes: number;
  last_sync: string | null;
}

export interface SyncLogEntry {
  id: number;
  account_id: number | null;
  list_id: number | null;
  task_id: number | null;
  operation: string;
  status: string;
  message: string | null;
  http_status: number | null;
  timestamp: string;
}
