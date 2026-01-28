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
  <div class="flex items-center gap-2">
    <button
      class="btn btn-sm preset-outlined flex items-center gap-1"
      onclick={handleSync}
      disabled={syncing}
      title={status.last_sync ? `Last sync: ${formatRelativeTime(status.last_sync)}` : 'Never synced'}
    >
      {#if syncing}
        <span class="w-3.5 h-3.5 border-2 border-surface-300-700 border-t-primary-500 rounded-full animate-spin"></span>
        Syncing...
      {:else}
        Sync
        {#if status.pending_changes > 0}
          <span class="badge preset-filled-primary-500 text-xs min-w-5 h-5 px-1">
            {status.pending_changes}
          </span>
        {/if}
      {/if}
    </button>

    {#if error}
      <span class="text-xs text-error-500" title={error}>Sync failed</span>
    {:else if status.failed_changes > 0 && status.last_error}
      <span class="text-xs text-error-500" title={status.last_error}>{status.failed_changes} failed</span>
    {:else if syncSummary()}
      <span class="text-xs text-surface-500">{syncSummary()}</span>
    {/if}
  </div>
{/if}
