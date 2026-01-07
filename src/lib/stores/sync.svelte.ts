// Sync store for managing sync operations
import { syncAccount } from '$lib/api';
import type { AccountSyncResult } from '$lib/types';

interface SyncState {
  syncing: boolean;
  syncingAccountId: number | null;
  lastResult: AccountSyncResult | null;
  error: string | null;
}

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
  };
}

export const syncStore = createSyncStore();
