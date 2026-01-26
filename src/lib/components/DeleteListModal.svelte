<script lang="ts">
  import { getTaskCount } from '$lib/api';
  import { listStore } from '$lib/stores/lists.svelte';
  import type { TaskList } from '$lib/types';

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

  const isSynced = $derived(!!list.caldav_url);
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={handleBackdropClick}>
  <div class="modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
    <div class="modal-header">
      <h2 id="modal-title">Delete List</h2>
      <button class="close-btn" onclick={onClose} aria-label="Close">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      {#if loading}
        <p class="loading">Loading...</p>
      {:else if step === 'confirm'}
        <p class="message">
          Are you sure you want to delete <strong>{list.name}</strong>?
        </p>
        {#if isSynced}
          <p class="sync-warning">
            <svg viewBox="0 0 24 24" fill="currentColor" class="warning-icon">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
            </svg>
            This list is synced to a CalDAV server and will also be deleted from the server.
          </p>
        {/if}
        {#if error}
          <p class="error">{error}</p>
        {/if}
        <div class="button-group">
          <button class="btn-secondary" onclick={onClose} disabled={deleting}>
            Cancel
          </button>
          <button class="btn-danger" onclick={handleFirstConfirm} disabled={deleting}>
            {deleting ? 'Deleting...' : 'Delete'}
          </button>
        </div>
      {:else}
        <p class="message warning-text">
          <svg viewBox="0 0 24 24" fill="currentColor" class="warning-icon large">
            <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
          </svg>
          This list contains <strong>{taskCount}</strong> {taskCount === 1 ? 'task' : 'tasks'} that will also be deleted.
        </p>
        <p class="secondary-message">This action cannot be undone.</p>
        {#if error}
          <p class="error">{error}</p>
        {/if}
        <div class="button-group">
          <button class="btn-secondary" onclick={() => step = 'confirm'} disabled={deleting}>
            Back
          </button>
          <button class="btn-danger" onclick={handleDelete} disabled={deleting}>
            {deleting ? 'Deleting...' : 'Delete Forever'}
          </button>
        </div>
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
    padding: env(safe-area-inset-top, 0) env(safe-area-inset-right, 0) env(safe-area-inset-bottom, 0) env(safe-area-inset-left, 0);
  }

  .modal {
    background: var(--bg-primary);
    border-radius: 12px;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
    width: 90%;
    max-width: 400px;
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
    padding: 1.5rem;
  }

  .loading {
    text-align: center;
    color: var(--text-secondary);
    padding: 1rem 0;
  }

  .message {
    margin: 0 0 1rem;
    font-size: 1rem;
    line-height: 1.5;
  }

  .message strong {
    color: var(--text-primary);
  }

  .warning-text {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    color: var(--error-color, #ef4444);
  }

  .warning-icon {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    color: var(--warning-color, #f59e0b);
  }

  .warning-icon.large {
    width: 24px;
    height: 24px;
    color: var(--error-color, #ef4444);
  }

  .sync-warning {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    margin: 0 0 1rem;
    padding: 0.75rem;
    background: var(--warning-bg, #fef3c7);
    border-radius: 8px;
    font-size: 0.875rem;
    color: var(--warning-text, #92400e);
  }

  .secondary-message {
    margin: 0 0 1rem;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .error {
    margin: 0 0 1rem;
    padding: 0.75rem;
    background: var(--error-bg, #fee2e2);
    border-radius: 8px;
    font-size: 0.875rem;
    color: var(--error-color, #ef4444);
  }

  .button-group {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
  }

  .button-group button {
    padding: 0.75rem 1.25rem;
    font-size: 0.875rem;
    font-weight: 500;
    border-radius: 8px;
    cursor: pointer;
    border: none;
    min-height: 44px;
  }

  .btn-secondary {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .btn-danger {
    background: var(--error-color, #ef4444);
    color: white;
  }

  .btn-danger:hover:not(:disabled) {
    background: #dc2626;
  }

  .button-group button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Mobile responsive */
  @media (max-width: 768px) {
    .modal {
      margin: 1rem;
    }

    .button-group {
      flex-direction: column-reverse;
    }

    .button-group button {
      width: 100%;
    }
  }
</style>
