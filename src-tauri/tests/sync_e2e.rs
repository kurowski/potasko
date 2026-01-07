//! End-to-end tests for CalDAV sync.
//!
//! These tests run against a local Radicale server (localhost:5232).
//! Run with: cargo test --test sync_e2e -- --test-threads=1

mod common;

use common::TestContext;
use predicates::prelude::*;

/// Test initial download of tasks from a CalDAV calendar.
#[test]
fn test_initial_download() {
    let ctx = TestContext::new();

    // 1. Create calendar on Radicale
    ctx.create_calendar().expect("Failed to create calendar");

    // 2. Create some VTODOs on the server
    let uid1 = "task-1-download";
    let uid2 = "task-2-download";
    ctx.create_server_vtodo(uid1, "Server Task 1")
        .expect("Failed to create VTODO 1");
    ctx.create_server_vtodo(uid2, "Server Task 2")
        .expect("Failed to create VTODO 2");

    // 3. Add account via CLI (this also initializes the database)
    ctx.add_account().expect("Failed to add account");

    // 4. Sync account - discovers calendar, imports as list, downloads tasks
    let list_id = ctx.sync_account(1).expect("Failed to sync account");

    // 5. Verify tasks are in local database
    ctx.cli()
        .args(["--format", "json", "task", "list", "--list", &list_id.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Server Task 1"))
        .stdout(predicate::str::contains("Server Task 2"));
}

/// Test pushing a new task to the server.
#[test]
fn test_push_new_task() {
    let ctx = TestContext::new();

    // 1. Create calendar on Radicale
    ctx.create_calendar().expect("Failed to create calendar");

    // 2. Add account
    ctx.add_account().expect("Failed to add account");

    // 3. Sync account to discover and import calendar
    let list_id = ctx.sync_account(1).expect("Failed to sync account");

    // 4. Add a task locally
    ctx.cli()
        .args(["task", "add", "Local New Task", "--list", &list_id.to_string()])
        .assert()
        .success();

    // 5. Sync
    ctx.cli()
        .args(["sync", "list", &list_id.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("pushed 1 new"));

    // 6. Verify sync status shows no pending changes
    ctx.cli()
        .args(["sync", "status", &list_id.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pending changes: 0"));
}

/// Test pushing an updated task to the server.
/// NOTE: This test documents a known bug where hrefs mismatch between push (full URL) and pull (relative path).
/// The task ID changes after sync due to recreate. We work around by finding the task after sync.
#[test]
fn test_push_update_task() {
    let ctx = TestContext::new();

    // 1. Setup
    ctx.create_calendar().expect("Failed to create calendar");
    ctx.add_account().expect("Failed to add account");
    let list_id = ctx.sync_account(1).expect("Failed to sync account");

    // 2. Create a task
    ctx.cli()
        .args(["task", "add", "Update Test Task", "--list", &list_id.to_string()])
        .assert()
        .success();

    // 3. Sync (push to server)
    ctx.cli()
        .args(["sync", "list", &list_id.to_string()])
        .assert()
        .success();

    // 4. Get the task ID from JSON output (task may have been recreated due to href mismatch bug)
    let list_output = ctx
        .cli()
        .args(["--format", "json", "task", "list", "--list", &list_id.to_string()])
        .output()
        .expect("Failed to list tasks");
    let tasks_json = String::from_utf8_lossy(&list_output.stdout);

    // Parse JSON to find the task ID
    let task_id: i64 = tasks_json
        .lines()
        .find(|line| line.contains("\"id\":"))
        .and_then(|line| {
            line.split(':')
                .nth(1)
                .and_then(|s| s.trim().trim_end_matches(',').parse().ok())
        })
        .expect("Failed to find task ID in JSON output");

    // 5. Complete the task (this marks it as changed)
    ctx.cli()
        .args(["task", "complete", &task_id.to_string()])
        .assert()
        .success();

    // 6. Sync again - should push the update
    ctx.cli()
        .args(["sync", "list", &list_id.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("pushed 1 updated"));
}

/// Test that completing a task syncs properly.
#[test]
fn test_push_complete_task() {
    let ctx = TestContext::new();

    // 1. Setup with server task
    ctx.create_calendar().expect("Failed to create calendar");
    let uid = "task-complete-test";
    ctx.create_server_vtodo(uid, "Task To Complete")
        .expect("Failed to create VTODO");

    // 2. Add account and sync - discovers calendar, imports list, downloads task
    ctx.add_account().expect("Failed to add account");
    let list_id = ctx.sync_account(1).expect("Failed to sync account");

    // 3. Get task ID (first task in the list)
    let list_output = ctx
        .cli()
        .args(["--format", "json", "task", "list", "--list", &list_id.to_string()])
        .output()
        .expect("Failed to list tasks");
    let tasks_json = String::from_utf8_lossy(&list_output.stdout);
    let task_id: i64 = tasks_json
        .lines()
        .find(|line| line.contains("\"id\":"))
        .and_then(|line| {
            line.split(':')
                .nth(1)
                .and_then(|s| s.trim().trim_end_matches(',').parse().ok())
        })
        .expect("Failed to find task ID");

    // 4. Complete it locally
    ctx.cli()
        .args(["task", "complete", &task_id.to_string()])
        .assert()
        .success();

    // 5. Sync
    ctx.cli()
        .args(["sync", "list", &list_id.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("pushed 1 updated"));

    // 6. Verify server has completed status
    let vtodo = ctx
        .get_server_vtodo(uid)
        .expect("Failed to get VTODO")
        .expect("VTODO should exist");
    assert!(
        vtodo.contains("STATUS:COMPLETED"),
        "Server VTODO should have COMPLETED status"
    );
}

/// Test pulling changes from the server.
#[test]
fn test_pull_server_changes() {
    let ctx = TestContext::new();

    // 1. Setup with synced task
    ctx.create_calendar().expect("Failed to create calendar");
    let uid = "task-pull-test";
    ctx.create_server_vtodo(uid, "Original Server Title")
        .expect("Failed to create VTODO");

    // 2. Add account and sync - discovers, imports, downloads
    ctx.add_account().expect("Failed to add account");
    let list_id = ctx.sync_account(1).expect("Failed to sync account");

    // 3. Update on server directly
    ctx.update_server_vtodo(uid, "Updated Server Title")
        .expect("Failed to update VTODO");

    // 4. Sync again
    ctx.cli()
        .args(["sync", "list", &list_id.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("pulled 1 updated"));

    // 5. Verify local task has new title
    ctx.cli()
        .args(["--format", "json", "task", "list", "--list", &list_id.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated Server Title"));
}

/// Test pulling a deletion from the server.
#[test]
fn test_pull_server_deletion() {
    let ctx = TestContext::new();

    // 1. Setup with synced task
    ctx.create_calendar().expect("Failed to create calendar");
    let uid = "task-delete-test";
    ctx.create_server_vtodo(uid, "Task To Be Deleted")
        .expect("Failed to create VTODO");

    // 2. Add account and sync - discovers, imports, downloads
    ctx.add_account().expect("Failed to add account");
    let list_id = ctx.sync_account(1).expect("Failed to sync account");

    // 3. Delete on server
    ctx.delete_server_vtodo(uid)
        .expect("Failed to delete VTODO");

    // 4. Sync again
    ctx.cli()
        .args(["sync", "list", &list_id.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("pulled 1 deleted"));

    // 5. Verify task is gone locally
    ctx.cli()
        .args(["--format", "json", "task", "list", "--list", &list_id.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task To Be Deleted").not());
}

/// Test that unchanged collection skips pull (CTag optimization).
#[test]
fn test_ctag_skip_unchanged() {
    let ctx = TestContext::new();

    // 1. Setup with synced task
    ctx.create_calendar().expect("Failed to create calendar");
    ctx.create_server_vtodo("task-ctag-test", "CTag Test Task")
        .expect("Failed to create VTODO");

    // 2. Add account and sync - discovers, imports, downloads
    ctx.add_account().expect("Failed to add account");
    let list_id = ctx.sync_account(1).expect("Failed to sync account");

    // 3. Sync again without changes
    ctx.cli()
        .args(["sync", "list", &list_id.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("no changes"));
}

/// Test sync status command shows pending changes.
#[test]
fn test_sync_status_shows_pending() {
    let ctx = TestContext::new();

    // 1. Setup
    ctx.create_calendar().expect("Failed to create calendar");
    ctx.add_account().expect("Failed to add account");
    let list_id = ctx.sync_account(1).expect("Failed to sync account");

    // 2. Add a task locally (not synced yet)
    ctx.cli()
        .args(["task", "add", "Pending Task", "--list", &list_id.to_string()])
        .assert()
        .success();

    // 3. Check status shows pending change
    ctx.cli()
        .args(["sync", "status", &list_id.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pending changes: 1"));

    // 4. Sync
    ctx.cli()
        .args(["sync", "list", &list_id.to_string()])
        .assert()
        .success();

    // 5. Check status shows no pending changes
    ctx.cli()
        .args(["sync", "status", &list_id.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pending changes: 0"));
}
