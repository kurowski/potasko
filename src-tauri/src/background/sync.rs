//! Background sync scheduler.
//!
//! Runs periodic sync for all accounts every 5 minutes.
//!
//! Note: If the user is editing a task when sync pulls a newer version from
//! the server, the user's subsequent save will overwrite the server version
//! (last-write-wins). This is intentional - see PLAN.md "Known Challenges".

use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;
use tokio::time::interval;

use crate::sync::{emit_sync_completed, SyncEngine};

/// Sync interval in seconds (5 minutes).
const SYNC_INTERVAL_SECS: u64 = 300;

/// Initial delay before first background sync (30 seconds).
const INITIAL_DELAY_SECS: u64 = 30;

/// Start the background sync scheduler.
///
/// This spawns an async task that:
/// 1. Waits 30 seconds after app startup
/// 2. Syncs all accounts every 5 minutes
/// 3. Emits events for each synced list so the frontend can update
pub fn start_background_sync(app: AppHandle, pool: Arc<SqlitePool>) {
    tauri::async_runtime::spawn(async move {
        // Initial delay before first background sync
        tokio::time::sleep(Duration::from_secs(INITIAL_DELAY_SECS)).await;

        let mut ticker = interval(Duration::from_secs(SYNC_INTERVAL_SECS));

        loop {
            ticker.tick().await;

            // Get all accounts
            let accounts = match crate::core::accounts::get_accounts(&pool).await {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Background sync: failed to get accounts: {}", e);
                    continue;
                }
            };

            if accounts.is_empty() {
                continue;
            }

            println!("Background sync: syncing {} account(s)", accounts.len());

            let engine = SyncEngine::new(SqlitePool::clone(&pool));

            for account in accounts {
                let result = engine.sync_account(account.id).await;

                // Emit event for each synced list
                for list_result in &result.list_results {
                    emit_sync_completed(&app, list_result);
                }

                if result.success {
                    println!(
                        "Background sync: account {} synced ({} lists)",
                        account.name,
                        result.list_results.len()
                    );
                } else if let Some(error) = &result.error {
                    eprintln!("Background sync: account {} failed: {}", account.name, error);
                }
            }
        }
    });
}
