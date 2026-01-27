<script lang="ts">
  import { getSyncLog } from '$lib/api';
  import type { SyncLogEntry } from '$lib/types';
  import {
    ComposedModal,
    ModalHeader,
    ModalBody,
    DataTable,
    Tag,
    Button,
    InlineLoading,
  } from 'carbon-components-svelte';

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

  function formatTimestamp(ts: string): string {
    try {
      const date = new Date(ts);
      return date.toLocaleString();
    } catch {
      return ts;
    }
  }

  function getStatusTagType(status: string): 'green' | 'red' | 'warm-gray' {
    switch (status) {
      case 'success': return 'green';
      case 'error': return 'red';
      case 'conflict': return 'warm-gray';
      default: return 'warm-gray';
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

  function formatDetails(entry: SyncLogEntry): string {
    const parts = [];
    if (entry.message) parts.push(entry.message);
    if (entry.list_id) parts.push(`List: ${entry.list_id}`);
    if (entry.task_id) parts.push(`Task: ${entry.task_id}`);
    if (entry.http_status) parts.push(`HTTP: ${entry.http_status}`);
    return parts.join(' · ');
  }

  // Transform entries for DataTable
  const tableRows = $derived(entries.map(entry => ({
    id: String(entry.id),
    time: formatTimestamp(entry.timestamp),
    operation: getOperationLabel(entry.operation),
    status: entry.status,
    details: formatDetails(entry),
  })));

  const headers = [
    { key: 'time', value: 'Time' },
    { key: 'operation', value: 'Operation' },
    { key: 'status', value: 'Status' },
    { key: 'details', value: 'Details' },
  ];
</script>

<ComposedModal open={true} on:close={onClose} size="lg">
  <ModalHeader title="Sync Log" />
  <ModalBody hasScrollingContent>
    {#if loading && entries.length === 0}
      <InlineLoading description="Loading sync log..." />
    {:else if error}
      <p class="error">{error}</p>
    {:else if entries.length === 0}
      <p class="empty">No sync operations logged yet.</p>
    {:else}
      <DataTable
        {headers}
        rows={tableRows}
        size="short"
      >
        <svelte:fragment slot="cell" let:cell let:row>
          {#if cell.key === 'status'}
            <Tag size="sm" type={getStatusTagType(cell.value)}>{cell.value}</Tag>
          {:else}
            {cell.value}
          {/if}
        </svelte:fragment>
      </DataTable>

      {#if hasMore}
        <div class="load-more">
          <Button
            kind="tertiary"
            size="small"
            on:click={() => loadEntries(true)}
            disabled={loading}
          >
            {loading ? 'Loading...' : 'Load More'}
          </Button>
        </div>
      {/if}
    {/if}
  </ModalBody>
</ComposedModal>

<style>
  .error {
    text-align: center;
    padding: 2rem;
    color: var(--cds-danger, #da1e28);
  }

  .empty {
    text-align: center;
    padding: 2rem;
    color: var(--cds-text-secondary, var(--text-secondary));
  }

  .load-more {
    display: flex;
    justify-content: center;
    padding: 1rem 0;
  }
</style>
