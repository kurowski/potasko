<script lang="ts">
  import type { SyncResult, ListSyncStatus } from '$lib/types';
  import { syncList, getSyncStatus } from '$lib/api';

  interface Props {
    listId: number;
  }

  let { listId }: Props = $props();

  let status: ListSyncStatus | null = $state(null);
  let syncing = $state(false);
  let error: string | null = $state(null);
  let lastResult: SyncResult | null = $state(null);

  // Load sync status
  async function loadStatus() {
    try {
      status = await getSyncStatus(listId);
    } catch (e) {
      console.error('Failed to load sync status:', e);
    }
  }

  // Sync the list
  async function handleSync() {
    if (syncing) return;

    syncing = true;
    error = null;

    try {
      lastResult = await syncList(listId);
      if (!lastResult.success) {
        error = lastResult.error ?? 'Sync failed';
      }
      // Refresh status after sync
      await loadStatus();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      syncing = false;
    }
  }

  // Load status on mount and when listId changes
  $effect(() => {
    if (listId) {
      loadStatus();
      // Reset state when list changes
      error = null;
      lastResult = null;
    }
  });

  // Format relative time
  function formatRelativeTime(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays}d ago`;
  }

  // Summary of last sync result
  const syncSummary = $derived(() => {
    if (!lastResult?.success) return null;
    const s = lastResult.stats;
    const parts: string[] = [];
    if (s.pushed_created) parts.push(`+${s.pushed_created}`);
    if (s.pushed_updated) parts.push(`~${s.pushed_updated}`);
    if (s.pushed_deleted) parts.push(`-${s.pushed_deleted}`);
    if (s.pulled_created) parts.push(`+${s.pulled_created}`);
    if (s.pulled_updated) parts.push(`~${s.pulled_updated}`);
    if (s.pulled_deleted) parts.push(`-${s.pulled_deleted}`);
    if (parts.length === 0) return 'No changes';
    return parts.join(' ');
  });
</script>

{#if status?.has_caldav}
  <div class="sync-status">
    <button
      class="sync-button"
      onclick={handleSync}
      disabled={syncing}
      title={status.last_sync ? `Last sync: ${formatRelativeTime(status.last_sync)}` : 'Never synced'}
    >
      {#if syncing}
        <span class="spinner"></span>
        Syncing...
      {:else}
        Sync
        {#if status.pending_changes > 0}
          <span class="pending-badge">{status.pending_changes}</span>
        {/if}
      {/if}
    </button>

    {#if error}
      <span class="sync-error" title={error}>Sync failed</span>
    {:else if status.failed_changes > 0 && status.last_error}
      <span class="sync-error" title={status.last_error}>{status.failed_changes} failed</span>
    {:else if syncSummary()}
      <span class="sync-summary">{syncSummary()}</span>
    {/if}
  </div>
{/if}

<style>
  .sync-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .sync-button {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    cursor: pointer;
    transition: background-color 0.15s, border-color 0.15s;
  }

  .sync-button:hover:not(:disabled) {
    background: var(--bg-hover);
    border-color: var(--border-hover);
  }

  .sync-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .pending-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.25rem;
    height: 1.25rem;
    padding: 0 0.25rem;
    font-size: 0.75rem;
    font-weight: 500;
    background: var(--accent-color, #3b82f6);
    color: white;
    border-radius: 10px;
  }

  .spinner {
    width: 0.875rem;
    height: 0.875rem;
    border: 2px solid var(--border-color);
    border-top-color: var(--accent-color, #3b82f6);
    border-radius: 50%;
    animation: spin 0.75s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .sync-error {
    font-size: 0.75rem;
    color: var(--error-color, #ef4444);
  }

  .sync-summary {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }
</style>
