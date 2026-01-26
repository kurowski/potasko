<script lang="ts">
  import type { TaskList } from '$lib/types';
  import { listStore } from '$lib/stores/lists.svelte';
  import { accountStore } from '$lib/stores/accounts.svelte';
  import { syncStore } from '$lib/stores/sync.svelte';
  import DeleteListModal from './DeleteListModal.svelte';

  interface Props {
    showSettings?: boolean;
    onToggleSettings?: () => void;
    isMobile?: boolean;
    isOpen?: boolean;
    onClose?: () => void;
  }

  let { showSettings = false, onToggleSettings, isMobile = false, isOpen = false, onClose }: Props = $props();

  // Close sidebar when a navigation item is selected on mobile
  function handleNavigation(action: () => void) {
    action();
    if (isMobile && onClose) {
      onClose();
    }
  }

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
  let newListAccountId = $state<number | null>(null);
  let isAdding = $state(false);

  // Edit list state
  let editingList = $state<TaskList | null>(null);
  let editName = $state('');
  let editColor = $state('');

  // Delete modal state
  let listToDelete = $state<TaskList | null>(null);

  // Group lists by account
  const groupedLists = $derived(() => {
    const localLists: TaskList[] = [];
    const accountLists = new Map<number, TaskList[]>();

    for (const list of listStore.lists) {
      if (list.account_id === null) {
        localLists.push(list);
      } else {
        const existing = accountLists.get(list.account_id) || [];
        existing.push(list);
        accountLists.set(list.account_id, existing);
      }
    }

    return { localLists, accountLists };
  });

  // Get account name by ID
  function getAccountName(accountId: number): string {
    const account = accountStore.accounts.find(a => a.id === accountId);
    return account?.name ?? `Account #${accountId}`;
  }

  async function handleAddList(e: Event) {
    e.preventDefault();
    if (!newListName.trim()) return;

    try {
      await listStore.create({
        name: newListName.trim(),
        color: newListColor,
        account_id: newListAccountId,
      });
      newListName = '';
      newListColor = COLORS[0];
      newListAccountId = null;
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
      newListAccountId = null;
      editingList = null;
    }
  }

  function handleDeleteClick(list: TaskList, e: Event) {
    e.stopPropagation();
    listToDelete = list;
  }
</script>

<aside class="sidebar" class:mobile={isMobile} class:open={isOpen}>
  {#if isMobile}
    <div class="sidebar-header mobile-close-header">
      <h2>Menu</h2>
      <button class="close-btn" onclick={onClose} aria-label="Close menu">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>
  {/if}

  <div class="sidebar-header">
    <h2>Views</h2>
  </div>

  <nav class="special-nav">
    <button
      class="special-item"
      class:selected={listStore.selectedSpecialView === 'today'}
      onclick={() => handleNavigation(() => listStore.selectSpecial('today'))}
    >
      <svg class="special-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z"/>
      </svg>
      <span class="list-name">Today</span>
    </button>
    <button
      class="special-item"
      class:selected={listStore.selectedSpecialView === 'overdue'}
      onclick={() => handleNavigation(() => listStore.selectSpecial('overdue'))}
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
      {#if accountStore.accounts.length > 0}
        <select bind:value={newListAccountId} class="account-select">
          <option value={null}>Local only</option>
          {#each accountStore.accounts as account (account.id)}
            <option value={account.id}>{account.name}</option>
          {/each}
        </select>
      {/if}
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
        <button type="button" onclick={() => { isAdding = false; newListName = ''; newListColor = COLORS[0]; newListAccountId = null; }}>Cancel</button>
      </div>
    </form>
  {/if}

  <nav class="list-nav">
    <!-- Local lists section -->
    {#if groupedLists().localLists.length > 0}
      <div class="list-section-header">Local</div>
      {#each groupedLists().localLists as list (list.id)}
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
              onclick={() => handleNavigation(() => listStore.select(list.id))}
            >
              {list.name}
            </button>
            <button
              type="button"
              class="delete-btn"
              class:mobile={isMobile}
              onclick={(e) => handleDeleteClick(list, e)}
              title="Delete list"
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
            </button>
          </div>
        {/if}
      {/each}
    {/if}

    <!-- Account lists sections -->
    {#each [...groupedLists().accountLists.entries()] as [accountId, lists] (accountId)}
      <div class="list-section-header">{getAccountName(accountId)}</div>
      {#each lists as list (list.id)}
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
              onclick={() => handleNavigation(() => listStore.select(list.id))}
            >
              {list.name}
            </button>
            <button
              type="button"
              class="delete-btn"
              class:mobile={isMobile}
              onclick={(e) => handleDeleteClick(list, e)}
              title="Delete list"
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
            </button>
          </div>
        {/if}
      {/each}
    {/each}
  </nav>

  {#if listStore.error}
    <p class="error">{listStore.error}</p>
  {/if}

  <div class="sidebar-footer">
    {#if syncStore.syncing}
      <div class="sync-indicator">
        <svg class="spinning" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
        </svg>
        <span>Syncing...</span>
      </div>
    {/if}
    <button
      class="settings-btn"
      class:active={showSettings}
      onclick={() => handleNavigation(() => onToggleSettings?.())}
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M19.14 12.94c.04-.31.06-.63.06-.94 0-.31-.02-.63-.06-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/>
      </svg>
      <span>Settings</span>
    </button>
  </div>
</aside>

{#if listToDelete}
  <DeleteListModal list={listToDelete} onClose={() => listToDelete = null} />
{/if}

<style>
  .sidebar {
    width: 240px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  /* Mobile drawer styles */
  .sidebar.mobile {
    position: fixed;
    top: 0;
    left: 0;
    width: 280px;
    height: 100%;
    z-index: 100;
    transform: translateX(-100%);
    transition: transform 0.3s ease;
    border-right: none;
    box-shadow: 2px 0 8px rgba(0, 0, 0, 0.15);
  }

  .sidebar.mobile.open {
    transform: translateX(0);
  }

  .sidebar.mobile .sidebar-header {
    padding-top: calc(1rem + env(safe-area-inset-top, 0));
  }

  .mobile-close-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-primary);
    border-radius: 8px;
  }

  .close-btn:hover {
    background: var(--bg-hover);
  }

  .close-btn svg {
    width: 24px;
    height: 24px;
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

  .account-select {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.875rem;
    background: var(--bg-primary);
    color: var(--text-primary);
    min-height: 44px;
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

  .list-section-header {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-secondary);
    padding: 0.75rem 0.75rem 0.375rem;
    margin-top: 0.25rem;
  }

  .list-section-header:first-child {
    margin-top: 0;
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

  .delete-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    opacity: 0;
    flex-shrink: 0;
    transition: opacity 0.15s ease;
  }

  .delete-btn svg {
    width: 14px;
    height: 14px;
  }

  .list-item:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    background: var(--error-color, #ef4444);
    color: white;
  }

  /* Mobile: always show delete button */
  .delete-btn.mobile {
    opacity: 1;
    width: 44px;
    height: 44px;
  }

  .delete-btn.mobile svg {
    width: 18px;
    height: 18px;
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

  .sync-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    font-size: 0.8125rem;
    color: var(--accent-color);
  }

  .sync-indicator svg {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
  }

  .spinning {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
