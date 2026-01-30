<script lang="ts">
  import { getSyncLog } from '$lib/api';
  import type { SyncLogEntry } from '$lib/types';
  import Dialog, { Title, Content, Actions } from '@smui/dialog';
  import Button from '@smui/button';
  import DataTable, { Head, Body, Row, Cell } from '@smui/data-table';
  import CircularProgress from '@smui/circular-progress';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  let entries = $state<SyncLogEntry[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let offset = $state(0);
  let hasMore = $state(true);
  let open = $state(true);

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

  function handleClose() {
    open = false;
    onClose();
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

<Dialog
  bind:open
  aria-labelledby="sync-log-dialog-title"
  aria-describedby="sync-log-dialog-content"
  surface$style="max-width: 700px; width: 90vw;"
  fullscreen
>
  <Title id="sync-log-dialog-title">Sync Log</Title>
  <Content id="sync-log-dialog-content">
    {#if loading && entries.length === 0}
      <div class="loading">
        <CircularProgress indeterminate />
        <p>Loading sync log...</p>
      </div>
    {:else if error}
      <div class="error-message">
        <span class="material-icons">error</span>
        {error}
      </div>
    {:else if entries.length === 0}
      <div class="empty-state">
        <span class="material-icons">history</span>
        <p>No sync operations logged yet.</p>
      </div>
    {:else}
      <div class="table-wrapper">
        <DataTable style="width: 100%;">
          <Head>
            <Row>
              <Cell>Time</Cell>
              <Cell>Operation</Cell>
              <Cell>Status</Cell>
              <Cell>Details</Cell>
            </Row>
          </Head>
          <Body>
            {#each entries as entry (entry.id)}
              <Row>
                <Cell class="timestamp-cell">{formatTimestamp(entry.timestamp)}</Cell>
                <Cell>{getOperationLabel(entry.operation)}</Cell>
                <Cell>
                  <span class="status {getStatusClass(entry.status)}">{entry.status}</span>
                </Cell>
                <Cell class="details-cell">
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
                </Cell>
              </Row>
            {/each}
          </Body>
        </DataTable>
      </div>

      {#if hasMore}
        <div class="load-more">
          <Button variant="text" onclick={() => loadEntries(true)} disabled={loading}>
            {loading ? 'Loading...' : 'Load More'}
          </Button>
        </div>
      {/if}
    {/if}
  </Content>
  <Actions>
    <Button onclick={handleClose}>Close</Button>
  </Actions>
</Dialog>

<style>
  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--app-spacing-4);
    padding: 2rem;
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .loading p {
    margin: 0;
  }

  .error-message {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--app-spacing-2);
    padding: 2rem;
    color: var(--mdc-theme-error, #dc2626);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--app-spacing-2);
    padding: 2rem;
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .empty-state .material-icons {
    font-size: 48px;
    opacity: 0.5;
  }

  .empty-state p {
    margin: 0;
  }

  .table-wrapper {
    overflow-x: auto;
  }

  :global(.timestamp-cell) {
    white-space: nowrap;
    font-size: var(--app-font-size-xs);
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .status {
    font-size: var(--app-font-size-2xs);
    padding: 0.125rem var(--app-spacing-2);
    border-radius: var(--app-radius-sm);
    text-transform: uppercase;
    font-weight: 500;
    white-space: nowrap;
  }

  .status-success {
    background: var(--app-color-status-success-bg);
    color: var(--app-color-status-success-text);
  }

  .status-error {
    background: var(--app-color-status-error-bg);
    color: var(--app-color-status-error-text);
  }

  .status-conflict {
    background: var(--app-color-status-warning-bg);
    color: var(--app-color-status-warning-text);
  }

  :global(.details-cell) {
    display: flex;
    flex-direction: column;
    gap: var(--app-spacing-1);
    max-width: 300px;
  }

  .message {
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
    word-break: break-word;
    font-size: 0.8125rem;
  }

  .meta {
    font-size: var(--app-font-size-2xs);
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
    opacity: 0.8;
  }

  .load-more {
    display: flex;
    justify-content: center;
    padding: var(--app-spacing-4) 0;
  }
</style>
