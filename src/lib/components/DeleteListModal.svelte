<script lang="ts">
  import { getTaskCount } from '$lib/api';
  import { listStore } from '$lib/stores/lists.svelte';
  import type { TaskList } from '$lib/types';
  import Dialog, { Title, Content, Actions } from '@smui/dialog';
  import Button from '@smui/button';
  import CircularProgress from '@smui/circular-progress';

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
  let open = $state(true);

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
      open = false;
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to delete list';
      deleting = false;
    }
  }

  function handleClose() {
    open = false;
    onClose();
  }

  const isSynced = $derived(!!list.caldav_url);
</script>

<Dialog
  bind:open
  aria-labelledby="delete-dialog-title"
  aria-describedby="delete-dialog-content"
  surface$style="max-width: 400px;"
>
  <Title id="delete-dialog-title">Delete List</Title>
  <Content id="delete-dialog-content">
    {#if loading}
      <div class="loading">
        <CircularProgress indeterminate />
      </div>
    {:else if step === 'confirm'}
      <p class="message">
        Are you sure you want to delete <strong>{list.name}</strong>?
      </p>
      {#if isSynced}
        <div class="sync-warning">
          <span class="material-icons">info</span>
          <span>This list is synced to a CalDAV server and will also be deleted from the server.</span>
        </div>
      {/if}
      {#if error}
        <div class="error-message">
          <span class="material-icons">error</span>
          {error}
        </div>
      {/if}
    {:else}
      <div class="warning-message">
        <span class="material-icons">warning</span>
        <div>
          <p class="warning-text">
            This list contains <strong>{taskCount}</strong> {taskCount === 1 ? 'task' : 'tasks'} that will also be deleted.
          </p>
          <p class="secondary-text">This action cannot be undone.</p>
        </div>
      </div>
      {#if error}
        <div class="error-message">
          <span class="material-icons">error</span>
          {error}
        </div>
      {/if}
    {/if}
  </Content>
  <Actions>
    {#if step === 'confirm'}
      <Button onclick={handleClose} disabled={deleting}>
        Cancel
      </Button>
      <Button onclick={handleFirstConfirm} disabled={deleting || loading}>
        <span class="delete-text">{deleting ? 'Deleting...' : 'Delete'}</span>
      </Button>
    {:else}
      <Button onclick={() => step = 'confirm'} disabled={deleting}>
        Back
      </Button>
      <Button onclick={handleDelete} disabled={deleting}>
        <span class="delete-text">{deleting ? 'Deleting...' : 'Delete Forever'}</span>
      </Button>
    {/if}
  </Actions>
</Dialog>

<style>
  .loading {
    display: flex;
    justify-content: center;
    padding: var(--app-spacing-4) 0;
  }

  .message {
    margin: 0 0 var(--app-spacing-4);
    line-height: 1.5;
  }

  .sync-warning {
    display: flex;
    align-items: flex-start;
    gap: var(--app-spacing-3);
    padding: var(--app-spacing-3) var(--app-spacing-4);
    background: var(--app-color-status-warning-bg);
    border-radius: var(--app-radius-md);
    font-size: var(--app-font-size-sm);
    color: var(--app-color-status-warning-text);
  }

  .sync-warning .material-icons {
    font-size: 20px;
    flex-shrink: 0;
  }

  .warning-message {
    display: flex;
    align-items: flex-start;
    gap: var(--app-spacing-3);
    color: var(--mdc-theme-error, #dc2626);
  }

  .warning-message .material-icons {
    font-size: 24px;
    color: var(--mdc-theme-error, #dc2626);
    flex-shrink: 0;
  }

  .warning-text {
    margin: 0 0 var(--app-spacing-2);
    line-height: 1.5;
  }

  .secondary-text {
    margin: 0;
    font-size: var(--app-font-size-sm);
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: var(--app-spacing-2);
    padding: var(--app-spacing-3) var(--app-spacing-4);
    background: var(--app-color-status-error-bg);
    border-radius: var(--app-radius-md);
    font-size: var(--app-font-size-sm);
    color: var(--app-color-status-error-text);
    margin-top: var(--app-spacing-4);
  }

  .error-message .material-icons {
    font-size: 20px;
    flex-shrink: 0;
  }

  .delete-text {
    color: var(--mdc-theme-error, #dc2626);
  }
</style>
