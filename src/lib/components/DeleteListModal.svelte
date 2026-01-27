<script lang="ts">
  import { getTaskCount } from '$lib/api';
  import { listStore } from '$lib/stores/lists.svelte';
  import type { TaskList } from '$lib/types';
  import {
    ComposedModal,
    ModalHeader,
    ModalBody,
    ModalFooter,
    Button,
    InlineNotification,
    InlineLoading,
  } from 'carbon-components-svelte';
  import { Warning, WarningAlt } from 'carbon-icons-svelte';

  interface Props {
    list: TaskList;
    onClose: () => void;
  }

  let { list, onClose }: Props = $props();

  let step = $state<'confirm' | 'warning'>('confirm');
  let taskCount = $state(0);
  let loading = $state(true);
  let deleting = $state(false);
  let error = $state<string | null>(null);

  // Load task count on mount
  $effect(() => {
    loadTaskCount();
  });

  async function loadTaskCount() {
    try {
      loading = true;
      taskCount = await getTaskCount(list.id);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load task count';
    } finally {
      loading = false;
    }
  }

  function handleFirstConfirm() {
    if (taskCount > 0) {
      step = 'warning';
    } else {
      handleDelete();
    }
  }

  async function handleDelete() {
    try {
      deleting = true;
      error = null;
      await listStore.remove(list.id);
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to delete list';
      deleting = false;
    }
  }

  const isSynced = $derived(!!list.caldav_url);
</script>

<ComposedModal open={true} on:close={onClose} danger>
  <ModalHeader title="Delete List" />
  <ModalBody>
    {#if loading}
      <InlineLoading description="Loading..." />
    {:else if step === 'confirm'}
      <p>
        Are you sure you want to delete <strong>{list.name}</strong>?
      </p>
      {#if isSynced}
        <InlineNotification
          kind="warning"
          title="Synced list"
          subtitle="This list is synced to a CalDAV server and will also be deleted from the server."
          hideCloseButton
        />
      {/if}
      {#if error}
        <InlineNotification
          kind="error"
          title="Error"
          subtitle={error}
          hideCloseButton
        />
      {/if}
    {:else}
      <div class="warning-message">
        <Warning size={24} class="warning-icon" />
        <p>
          This list contains <strong>{taskCount}</strong> {taskCount === 1 ? 'task' : 'tasks'} that will also be deleted.
        </p>
      </div>
      <p class="secondary-message">This action cannot be undone.</p>
      {#if error}
        <InlineNotification
          kind="error"
          title="Error"
          subtitle={error}
          hideCloseButton
        />
      {/if}
    {/if}
  </ModalBody>
  <ModalFooter>
    {#if step === 'confirm'}
      <Button kind="secondary" on:click={onClose} disabled={deleting || loading}>
        Cancel
      </Button>
      <Button kind="danger" on:click={handleFirstConfirm} disabled={deleting || loading}>
        {deleting ? 'Deleting...' : 'Delete'}
      </Button>
    {:else}
      <Button kind="secondary" on:click={() => step = 'confirm'} disabled={deleting}>
        Back
      </Button>
      <Button kind="danger" on:click={handleDelete} disabled={deleting}>
        {deleting ? 'Deleting...' : 'Delete Forever'}
      </Button>
    {/if}
  </ModalFooter>
</ComposedModal>

<style>
  p {
    margin: 0 0 1rem;
  }

  p strong {
    font-weight: 600;
  }

  .warning-message {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    color: var(--cds-danger, #da1e28);
    margin-bottom: 1rem;
  }

  .warning-message :global(.warning-icon) {
    flex-shrink: 0;
  }

  .warning-message p {
    margin: 0;
  }

  .secondary-message {
    font-size: 0.875rem;
    color: var(--cds-text-secondary, var(--text-secondary));
  }
</style>
