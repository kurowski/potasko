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
<div
  class="fixed inset-0 bg-black/50 flex items-center justify-center z-[1000] p-[env(safe-area-inset-top,0)] p-[env(safe-area-inset-right,0)] p-[env(safe-area-inset-bottom,0)] p-[env(safe-area-inset-left,0)]"
  onclick={handleBackdropClick}
>
  <div class="card bg-surface-50-950 rounded-xl shadow-xl w-[90%] max-w-md flex flex-col" role="dialog" aria-modal="true" aria-labelledby="modal-title">
    <header class="flex justify-between items-center p-4 px-6 border-b border-surface-300-700">
      <h2 id="modal-title" class="m-0 text-xl font-semibold">Delete List</h2>
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

    <div class="p-6">
      {#if loading}
        <p class="text-center text-surface-500 py-4">Loading...</p>
      {:else if step === 'confirm'}
        <p class="m-0 mb-4 text-base leading-relaxed">
          Are you sure you want to delete <strong>{list.name}</strong>?
        </p>
        {#if isSynced}
          <div class="flex items-start gap-2 mb-4 p-3 bg-warning-500/20 rounded-lg text-sm text-warning-700 dark:text-warning-300">
            <svg class="w-5 h-5 shrink-0 text-warning-500" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
            </svg>
            <span>This list is synced to a CalDAV server and will also be deleted from the server.</span>
          </div>
        {/if}
        {#if error}
          <p class="m-0 mb-4 p-3 bg-error-500/20 rounded-lg text-sm text-error-500">{error}</p>
        {/if}
        <div class="flex gap-3 justify-end flex-col-reverse md:flex-row">
          <button class="btn preset-outlined min-h-11" onclick={onClose} disabled={deleting}>
            Cancel
          </button>
          <button class="btn preset-filled-error-500 min-h-11" onclick={handleFirstConfirm} disabled={deleting}>
            {deleting ? 'Deleting...' : 'Delete'}
          </button>
        </div>
      {:else}
        <div class="flex items-start gap-3 mb-4 text-error-500">
          <svg class="w-6 h-6 shrink-0" viewBox="0 0 24 24" fill="currentColor">
            <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
          </svg>
          <p class="m-0">
            This list contains <strong>{taskCount}</strong> {taskCount === 1 ? 'task' : 'tasks'} that will also be deleted.
          </p>
        </div>
        <p class="m-0 mb-4 text-sm text-surface-500">This action cannot be undone.</p>
        {#if error}
          <p class="m-0 mb-4 p-3 bg-error-500/20 rounded-lg text-sm text-error-500">{error}</p>
        {/if}
        <div class="flex gap-3 justify-end flex-col-reverse md:flex-row">
          <button class="btn preset-outlined min-h-11" onclick={() => step = 'confirm'} disabled={deleting}>
            Back
          </button>
          <button class="btn preset-filled-error-500 min-h-11" onclick={handleDelete} disabled={deleting}>
            {deleting ? 'Deleting...' : 'Delete Forever'}
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>
