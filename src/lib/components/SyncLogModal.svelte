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
      case 'success': return 'status-success';
      case 'error': return 'status-error';
      case 'conflict': return 'status-conflict';
      default: return '';
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
<div class="modal-backdrop" onclick={handleBackdropClick}>
  <div class="modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
    <div class="modal-header">
      <h2 id="modal-title">Sync Log</h2>
      <button class="close-btn" onclick={onClose} aria-label="Close">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      {#if loading && entries.length === 0}
        <p class="loading">Loading sync log...</p>
      {:else if error}
        <p class="error">{error}</p>
      {:else if entries.length === 0}
        <p class="empty">No sync operations logged yet.</p>
      {:else}
        <div class="table-wrapper">
          <table class="log-table">
            <thead>
              <tr>
                <th>Time</th>
                <th>Operation</th>
                <th>Status</th>
                <th>Details</th>
              </tr>
            </thead>
            <tbody>
              {#each entries as entry (entry.id)}
                <tr>
                  <td class="timestamp">{formatTimestamp(entry.timestamp)}</td>
                  <td>{getOperationLabel(entry.operation)}</td>
                  <td><span class="status {getStatusClass(entry.status)}">{entry.status}</span></td>
                  <td class="details">
                    {#if entry.message}
                      <span class="message">{entry.message}</span>
                    {/if}
                    {#if entry.list_id || entry.task_id || entry.http_status}
                      <span class="meta">
                        {#if entry.list_id}List: {entry.list_id}{/if}
                        {#if entry.task_id}{entry.list_id ? ' · ' : ''}Task: {entry.task_id}{/if}
                        {#if entry.http_status}{(entry.list_id || entry.task_id) ? ' · ' : ''}HTTP: {entry.http_status}{/if}
                      </span>
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>

        {#if hasMore}
          <button
            class="load-more-btn"
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

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--bg-primary);
    border-radius: 12px;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
    width: 90%;
    max-width: 600px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--border-color);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .close-btn {
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
  }

  .close-btn svg {
    width: 20px;
    height: 20px;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .modal-body {
    padding: 1rem 1.5rem;
    overflow-y: auto;
    flex: 1;
  }

  .loading,
  .empty,
  .error {
    text-align: center;
    padding: 2rem;
    color: var(--text-secondary);
  }

  .error {
    color: var(--error-color, #ef4444);
  }

  .table-wrapper {
    overflow-x: auto;
  }

  .log-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8125rem;
  }

  .log-table th,
  .log-table td {
    padding: 0.5rem 0.75rem;
    text-align: left;
    border-bottom: 1px solid var(--border-color);
  }

  .log-table th {
    font-weight: 600;
    font-size: 0.75rem;
    text-transform: uppercase;
    color: var(--text-secondary);
    background: var(--bg-secondary);
    position: sticky;
    top: 0;
  }

  .log-table tbody tr:hover {
    background: var(--bg-hover);
  }

  .timestamp {
    white-space: nowrap;
    color: var(--text-secondary);
    font-size: 0.75rem;
  }

  .status {
    font-size: 0.6875rem;
    padding: 0.125rem 0.5rem;
    border-radius: 4px;
    text-transform: uppercase;
    font-weight: 500;
    white-space: nowrap;
  }

  .status-success {
    background: var(--priority-low-bg, #dcfce7);
    color: var(--priority-low-text, #166534);
  }

  .status-error {
    background: var(--priority-high-bg, #fee2e2);
    color: var(--priority-high-text, #991b1b);
  }

  .status-conflict {
    background: var(--priority-medium-bg, #fef3c7);
    color: var(--priority-medium-text, #92400e);
  }

  .details {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .message {
    color: var(--text-secondary);
    word-break: break-word;
  }

  .meta {
    font-size: 0.6875rem;
    color: var(--text-secondary);
    opacity: 0.8;
  }

  .load-more-btn {
    display: block;
    width: 100%;
    padding: 0.75rem;
    margin-top: 1rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    color: var(--text-primary);
  }

  .load-more-btn:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .load-more-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
