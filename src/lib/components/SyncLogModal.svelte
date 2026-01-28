<script lang="ts">
  import { getSyncLog } from '$lib/api';
  import type { SyncLogEntry } from '$lib/types';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  let entries = $state<SyncLogEntry[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let offset = $state(0);
  let hasMore = $state(true);

  const PAGE_SIZE = 50;

  async function loadEntries(loadMore = false) {
    try {
      loading = true;
      error = null;
      const newEntries = await getSyncLog(PAGE_SIZE, loadMore ? offset : 0);

      if (loadMore) {
        entries = [...entries, ...newEntries];
      } else {
        entries = newEntries;
      }

      offset = entries.length;
      hasMore = newEntries.length === PAGE_SIZE;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load sync log';
    } finally {
      loading = false;
    }
  }

  // Load on mount
  $effect(() => {
    loadEntries();
  });

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      onClose();
    }
  }

  function formatTimestamp(ts: string): string {
    try {
      const date = new Date(ts);
      return date.toLocaleString();
    } catch {
      return ts;
    }
  }

  function getStatusClass(status: string): string {
    switch (status) {
      case 'success': return 'preset-filled-success-500';
      case 'error': return 'preset-filled-error-500';
      case 'conflict': return 'preset-filled-warning-500';
      default: return 'preset-filled-surface-500';
    }
  }

  function getOperationLabel(operation: string): string {
    const labels: Record<string, string> = {
      'push_create': 'Push Create',
      'push_update': 'Push Update',
      'push_delete': 'Push Delete',
      'push_conflict': 'Push Conflict',
      'push_error': 'Push Error',
      'pull_create': 'Pull Create',
      'pull_update': 'Pull Update',
      'pull_delete': 'Pull Delete',
      'pull_parse_error': 'Pull Parse Error',
    };
    return labels[operation] || operation;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 bg-black/50 flex items-center justify-center z-[1000]"
  onclick={handleBackdropClick}
>
  <div class="card bg-surface-50-950 rounded-xl shadow-xl w-[90%] max-w-[600px] max-h-[80vh] flex flex-col" role="dialog" aria-modal="true" aria-labelledby="modal-title">
    <header class="flex justify-between items-center p-4 px-6 border-b border-surface-300-700">
      <h2 id="modal-title" class="m-0 text-xl font-semibold">Sync Log</h2>
      <button
        class="btn-icon preset-tonal"
        onclick={onClose}
        aria-label="Close"
      >
        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
        </svg>
      </button>
    </header>

    <div class="p-4 px-6 overflow-y-auto flex-1">
      {#if loading && entries.length === 0}
        <p class="text-center text-surface-500 py-8">Loading sync log...</p>
      {:else if error}
        <p class="text-center text-error-500 py-8">{error}</p>
      {:else if entries.length === 0}
        <p class="text-center text-surface-500 py-8">No sync operations logged yet.</p>
      {:else}
        <div class="overflow-x-auto">
          <table class="w-full border-collapse text-[0.8125rem]">
            <thead>
              <tr>
                <th class="p-2 px-3 text-left border-b border-surface-300-700 font-semibold text-xs uppercase text-surface-500 bg-surface-100-900 sticky top-0">Time</th>
                <th class="p-2 px-3 text-left border-b border-surface-300-700 font-semibold text-xs uppercase text-surface-500 bg-surface-100-900 sticky top-0">Operation</th>
                <th class="p-2 px-3 text-left border-b border-surface-300-700 font-semibold text-xs uppercase text-surface-500 bg-surface-100-900 sticky top-0">Status</th>
                <th class="p-2 px-3 text-left border-b border-surface-300-700 font-semibold text-xs uppercase text-surface-500 bg-surface-100-900 sticky top-0">Details</th>
              </tr>
            </thead>
            <tbody>
              {#each entries as entry (entry.id)}
                <tr class="hover:bg-surface-100-900">
                  <td class="p-2 px-3 border-b border-surface-300-700 whitespace-nowrap text-surface-500 text-xs">{formatTimestamp(entry.timestamp)}</td>
                  <td class="p-2 px-3 border-b border-surface-300-700">{getOperationLabel(entry.operation)}</td>
                  <td class="p-2 px-3 border-b border-surface-300-700">
                    <span class="badge text-[0.6875rem] px-2 py-0.5 uppercase font-medium whitespace-nowrap {getStatusClass(entry.status)}">
                      {entry.status}
                    </span>
                  </td>
                  <td class="p-2 px-3 border-b border-surface-300-700">
                    <div class="flex flex-col gap-1">
                      {#if entry.message}
                        <span class="text-surface-500 break-words">{entry.message}</span>
                      {/if}
                      {#if entry.list_id || entry.task_id || entry.http_status}
                        <span class="text-[0.6875rem] text-surface-500 opacity-80">
                          {#if entry.list_id}List: {entry.list_id}{/if}
                          {#if entry.task_id}{entry.list_id ? ' · ' : ''}Task: {entry.task_id}{/if}
                          {#if entry.http_status}{(entry.list_id || entry.task_id) ? ' · ' : ''}HTTP: {entry.http_status}{/if}
                        </span>
                      {/if}
                    </div>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>

        {#if hasMore}
          <button
            class="btn preset-outlined w-full mt-4"
            onclick={() => loadEntries(true)}
            disabled={loading}
          >
            {loading ? 'Loading...' : 'Load More'}
          </button>
        {/if}
      {/if}
    </div>
  </div>
</div>
