<script lang="ts">
  import type { TaskList } from '$lib/types';
  import { listStore } from '$lib/stores/lists.svelte';

  interface Props {
    showSettings?: boolean;
    onToggleSettings?: () => void;
  }

  let { showSettings = false, onToggleSettings }: Props = $props();

  // Predefined color palette
  const COLORS = [
    '#6b7280', // gray
    '#ef4444', // red
    '#f97316', // orange
    '#eab308', // yellow
    '#22c55e', // green
    '#14b8a6', // teal
    '#3b82f6', // blue
    '#8b5cf6', // purple
    '#ec4899', // pink
  ];

  // Add list state
  let newListName = $state('');
  let newListColor = $state(COLORS[0]);
  let isAdding = $state(false);

  // Edit list state
  let editingList = $state<TaskList | null>(null);
  let editName = $state('');
  let editColor = $state('');

  async function handleAddList(e: Event) {
    e.preventDefault();
    if (!newListName.trim()) return;

    try {
      await listStore.create({ name: newListName.trim(), color: newListColor });
      newListName = '';
      newListColor = COLORS[0];
      isAdding = false;
    } catch {
      // Error is handled by store
    }
  }

  function startEdit(list: TaskList) {
    editingList = list;
    editName = list.name;
    editColor = list.color ?? COLORS[0];
  }

  async function handleEditList(e: Event) {
    e.preventDefault();
    if (!editingList || !editName.trim()) return;

    try {
      await listStore.update(editingList.id, { name: editName.trim(), color: editColor });
      editingList = null;
    } catch {
      // Error is handled by store
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      isAdding = false;
      newListName = '';
      newListColor = COLORS[0];
      editingList = null;
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
    <form class="list-form" onsubmit={handleAddList} onkeydown={handleKeydown}>
      <input
        type="text"
        bind:value={newListName}
        placeholder="List name..."
        autofocus
      />
      <div class="color-picker">
        {#each COLORS as color}
          <button
            type="button"
            class="color-option"
            class:selected={newListColor === color}
            style:background-color={color}
            onclick={() => newListColor = color}
            title={color}
          ></button>
        {/each}
      </div>
      <div class="form-buttons">
        <button type="submit" class="primary" disabled={!newListName.trim()}>Add</button>
        <button type="button" onclick={() => { isAdding = false; newListName = ''; newListColor = COLORS[0]; }}>Cancel</button>
      </div>
    </form>
  {/if}

  <nav class="list-nav">
    {#each listStore.lists as list (list.id)}
      {#if editingList?.id === list.id}
        <form class="list-form inline" onsubmit={handleEditList} onkeydown={handleKeydown}>
          <input
            type="text"
            bind:value={editName}
            autofocus
          />
          <div class="color-picker">
            {#each COLORS as color}
              <button
                type="button"
                class="color-option"
                class:selected={editColor === color}
                style:background-color={color}
                onclick={() => editColor = color}
                title={color}
              ></button>
            {/each}
          </div>
          <div class="form-buttons">
            <button type="submit" class="primary" disabled={!editName.trim()}>Save</button>
            <button type="button" onclick={() => editingList = null}>Cancel</button>
          </div>
        </form>
      {:else}
        <div
          class="list-item"
          class:selected={list.id === listStore.selectedListId}
        >
          <button
            type="button"
            class="list-color-btn"
            style:background-color={list.color ?? '#6b7280'}
            onclick={() => startEdit(list)}
            title="Edit list"
          ></button>
          <button
            type="button"
            class="list-name-btn"
            onclick={() => listStore.select(list.id)}
          >
            {list.name}
          </button>
        </div>
      {/if}
    {/each}
  </nav>

  {#if listStore.error}
    <p class="error">{listStore.error}</p>
  {/if}

  <div class="sidebar-footer">
    <button
      class="settings-btn"
      class:active={showSettings}
      onclick={onToggleSettings}
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M19.14 12.94c.04-.31.06-.63.06-.94 0-.31-.02-.63-.06-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/>
      </svg>
      <span>Settings</span>
    </button>
  </div>
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

  .list-form {
    padding: 0.5rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    border-bottom: 1px solid var(--border-color);
  }

  .list-form.inline {
    padding: 0.5rem;
    border-bottom: none;
    background: var(--bg-hover);
    border-radius: 6px;
    margin-bottom: 0.25rem;
  }

  .list-form input {
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.875rem;
    background: var(--bg-primary);
  }

  .color-picker {
    display: flex;
    gap: 0.375rem;
    flex-wrap: wrap;
  }

  .color-option {
    width: 20px;
    height: 20px;
    border: 2px solid transparent;
    border-radius: 4px;
    cursor: pointer;
    padding: 0;
  }

  .color-option:hover {
    transform: scale(1.1);
  }

  .color-option.selected {
    border-color: var(--text-primary);
  }

  .form-buttons {
    display: flex;
    gap: 0.5rem;
  }

  .form-buttons button {
    padding: 0.375rem 0.75rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.875rem;
  }

  .form-buttons button.primary {
    background: var(--accent-color);
    color: white;
  }

  .form-buttons button.primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .form-buttons button:not(.primary) {
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
    gap: 0.5rem;
    padding: 0.375rem;
    border-radius: 6px;
    font-size: 0.875rem;
  }

  .list-item:hover {
    background: var(--bg-hover);
  }

  .list-item.selected {
    background: var(--bg-selected);
  }

  .list-color-btn {
    width: 14px;
    height: 14px;
    border-radius: 3px;
    flex-shrink: 0;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .list-color-btn:hover {
    transform: scale(1.15);
  }

  .list-name-btn {
    flex: 1;
    text-align: left;
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 0.875rem;
    padding: 0.25rem 0.375rem;
    border-radius: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .list-name-btn:hover {
    background: var(--bg-hover);
  }

  .error {
    padding: 0.5rem 1rem;
    color: var(--error-color);
    font-size: 0.75rem;
  }

  .sidebar-footer {
    margin-top: auto;
    padding: 0.5rem;
    border-top: 1px solid var(--border-color);
  }

  .settings-btn {
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
    color: var(--text-secondary);
  }

  .settings-btn svg {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
  }

  .settings-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .settings-btn.active {
    background: var(--bg-selected);
    color: var(--text-primary);
  }
</style>
