<script lang="ts">
  import { listStore } from '$lib/stores/lists.svelte';

  let newListName = $state('');
  let isAdding = $state(false);

  async function handleAddList(e: Event) {
    e.preventDefault();
    if (!newListName.trim()) return;

    try {
      await listStore.create({ name: newListName.trim() });
      newListName = '';
      isAdding = false;
    } catch {
      // Error is handled by store
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      isAdding = false;
      newListName = '';
    }
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <h2>Views</h2>
  </div>

  <nav class="special-nav">
    <button
      class="special-item"
      class:selected={listStore.selectedSpecialView === 'today'}
      onclick={() => listStore.selectSpecial('today')}
    >
      <svg class="special-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z"/>
      </svg>
      <span class="list-name">Today</span>
    </button>
    <button
      class="special-item"
      class:selected={listStore.selectedSpecialView === 'overdue'}
      onclick={() => listStore.selectSpecial('overdue')}
    >
      <svg class="special-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
      </svg>
      <span class="list-name">Overdue</span>
    </button>
  </nav>

  <div class="sidebar-header">
    <h2>Lists</h2>
    <button class="icon-btn" onclick={() => isAdding = true} title="Add list">+</button>
  </div>

  {#if isAdding}
    <form class="add-list-form" onsubmit={handleAddList}>
      <input
        type="text"
        bind:value={newListName}
        placeholder="List name..."
        onkeydown={handleKeydown}
        autofocus
      />
      <button type="submit" disabled={!newListName.trim()}>Add</button>
      <button type="button" onclick={() => { isAdding = false; newListName = ''; }}>Cancel</button>
    </form>
  {/if}

  <nav class="list-nav">
    {#each listStore.lists as list (list.id)}
      <button
        class="list-item"
        class:selected={list.id === listStore.selectedListId}
        onclick={() => listStore.select(list.id)}
      >
        <span class="list-color" style:background-color={list.color ?? '#6b7280'}></span>
        <span class="list-name">{list.name}</span>
      </button>
    {/each}
  </nav>

  {#if listStore.error}
    <p class="error">{listStore.error}</p>
  {/if}
</aside>

<style>
  .sidebar {
    width: 240px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid var(--border-color);
  }

  .sidebar-header h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .icon-btn {
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 1.25rem;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon-btn:hover {
    background: var(--bg-hover);
  }

  .add-list-form {
    padding: 0.5rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    border-bottom: 1px solid var(--border-color);
  }

  .add-list-form input {
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .add-list-form button {
    padding: 0.375rem 0.75rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.875rem;
  }

  .add-list-form button[type="submit"] {
    background: var(--accent-color);
    color: white;
  }

  .add-list-form button[type="submit"]:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .add-list-form button[type="button"] {
    background: transparent;
  }

  .special-nav {
    padding: 0.5rem;
    border-bottom: 1px solid var(--border-color);
  }

  .special-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.625rem 0.75rem;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 6px;
    text-align: left;
    font-size: 0.875rem;
  }

  .special-item:hover {
    background: var(--bg-hover);
  }

  .special-item.selected {
    background: var(--bg-selected);
  }

  .special-icon {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    color: var(--text-secondary);
  }

  .special-item.selected .special-icon {
    color: var(--text-primary);
  }

  .list-nav {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
  }

  .list-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.625rem 0.75rem;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 6px;
    text-align: left;
    font-size: 0.875rem;
  }

  .list-item:hover {
    background: var(--bg-hover);
  }

  .list-item.selected {
    background: var(--bg-selected);
  }

  .list-color {
    width: 12px;
    height: 12px;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .list-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .error {
    padding: 0.5rem 1rem;
    color: var(--error-color);
    font-size: 0.75rem;
  }
</style>
