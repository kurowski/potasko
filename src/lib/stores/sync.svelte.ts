// Sync store for managing sync operations
import { syncAccount } from '$lib/api';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { AccountSyncResult, SyncResult } from '$lib/types';

// Track event listeners for cleanup
let unlistenFns: UnlistenFn[] = [];

function createSyncStore() {
  let syncing = $state(false);
  let syncingAccountId = $state<number | null>(null);
  let lastResult = $state<AccountSyncResult | null>(null);
  let error = $state<string | null>(null);

  return {
    get syncing() { return syncing; },
    get syncingAccountId() { return syncingAccountId; },
    get lastResult() { return lastResult; },
    get error() { return error; },

    async syncAccount(accountId: number, onComplete?: () => void) {
      if (syncing) return;

      syncing = true;
      syncingAccountId = accountId;
      error = null;
      lastResult = null;

      try {
        const result = await syncAccount(accountId);
        lastResult = result;

        if (!result.success && result.error) {
          error = result.error;
        }

        // Call completion callback (e.g., to refresh lists)
        if (onComplete) {
          onComplete();
        }
      } catch (e) {
        error = e instanceof Error ? e.message : String(e);
      } finally {
        syncing = false;
        syncingAccountId = null;
      }
    },

    clearResult() {
      lastResult = null;
      error = null;
    },

    /**
     * Setup event listeners for sync events from the backend.
     * Call this in onMount and pass a callback to handle list sync completion.
     */
    async setupEventListeners(onListSynced: (listId: number) => void) {
      const unlisten = await listen<SyncResult>('sync:list-completed', (event) => {
        onListSynced(event.payload.list_id);
      });
      unlistenFns.push(unlisten);
    },

    /**
     * Cleanup event listeners. Call this in component cleanup/onDestroy.
     */
    cleanup() {
      unlistenFns.forEach(fn => fn());
      unlistenFns = [];
    },
  };
}

export const syncStore = createSyncStore();
