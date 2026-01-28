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

<aside
  class="w-60 bg-surface-100-900 border-r border-surface-300-700 flex flex-col h-full
         {isMobile ? 'fixed top-0 left-0 w-70 z-100 shadow-lg transition-transform duration-300' : ''}
         {isMobile && !isOpen ? '-translate-x-full' : ''}"
>
  {#if isMobile}
    <div class="flex justify-between items-center p-4 pt-[calc(1rem+var(--safe-area-top))] border-b border-surface-300-700">
      <h2 class="m-0 text-base font-semibold">Menu</h2>
      <button
        class="btn-icon preset-tonal"
        onclick={onClose}
        aria-label="Close menu"
      >
        <svg class="w-6 h-6" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>
  {/if}

  <div class="flex justify-between items-center p-4 border-b border-surface-300-700">
    <h2 class="m-0 text-base font-semibold">Views</h2>
  </div>

  <nav class="p-2 border-b border-surface-300-700">
    <button
      class="w-full flex items-center gap-3 px-3 py-2.5 rounded-md text-sm text-left transition-colors
             {listStore.selectedSpecialView === 'today' ? 'bg-surface-200-800' : 'hover:bg-surface-200-800'}"
      onclick={() => handleNavigation(() => listStore.selectSpecial('today'))}
    >
      <svg class="w-[18px] h-[18px] shrink-0 text-surface-500" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z"/>
      </svg>
      <span>Today</span>
    </button>
    <button
      class="w-full flex items-center gap-3 px-3 py-2.5 rounded-md text-sm text-left transition-colors
             {listStore.selectedSpecialView === 'overdue' ? 'bg-surface-200-800' : 'hover:bg-surface-200-800'}"
      onclick={() => handleNavigation(() => listStore.selectSpecial('overdue'))}
    >
      <svg class="w-[18px] h-[18px] shrink-0 text-surface-500" viewBox="0 0 24 24" fill="currentColor">
        <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
      </svg>
      <span>Overdue</span>
    </button>
  </nav>

  <div class="flex justify-between items-center p-4 border-b border-surface-300-700">
    <h2 class="m-0 text-base font-semibold">Lists</h2>
    <button
      class="btn-icon preset-tonal size-7"
      onclick={() => isAdding = true}
      title="Add list"
    >
      +
    </button>
  </div>

  {#if isAdding}
    <form class="p-2 px-4 flex flex-col gap-2 border-b border-surface-300-700" onsubmit={handleAddList} onkeydown={handleKeydown}>
      <input
        type="text"
        class="input text-sm"
        bind:value={newListName}
        placeholder="List name..."
        autofocus
      />
      {#if accountStore.accounts.length > 0}
        <select bind:value={newListAccountId} class="input text-sm min-h-11">
          <option value={null}>Local only</option>
          {#each accountStore.accounts as account (account.id)}
            <option value={account.id}>{account.name}</option>
          {/each}
        </select>
      {/if}
      <div class="flex gap-1.5 flex-wrap">
        {#each COLORS as color}
          <button
            type="button"
            class="w-5 h-5 rounded border-2 p-0 cursor-pointer transition-transform hover:scale-110
                   {newListColor === color ? 'border-surface-900 dark:border-surface-50' : 'border-transparent'}"
            style:background-color={color}
            onclick={() => newListColor = color}
            title={color}
          ></button>
        {/each}
      </div>
      <div class="flex gap-2">
        <button type="submit" class="btn btn-sm preset-filled-primary-500" disabled={!newListName.trim()}>Add</button>
        <button type="button" class="btn btn-sm preset-outlined" onclick={() => { isAdding = false; newListName = ''; newListColor = COLORS[0]; newListAccountId = null; }}>Cancel</button>
      </div>
    </form>
  {/if}

  <nav class="flex-1 overflow-y-auto p-2">
    <!-- Local lists section -->
    {#if groupedLists().localLists.length > 0}
      <div class="text-[0.6875rem] font-semibold uppercase tracking-wide text-surface-500 px-3 py-1.5 mt-1">Local</div>
      {#each groupedLists().localLists as list (list.id)}
        {#if editingList?.id === list.id}
          <form class="p-2 bg-surface-200-800 rounded-md mb-1 flex flex-col gap-2" onsubmit={handleEditList} onkeydown={handleKeydown}>
            <input
              type="text"
              class="input text-sm"
              bind:value={editName}
              autofocus
            />
            <div class="flex gap-1.5 flex-wrap">
              {#each COLORS as color}
                <button
                  type="button"
                  class="w-5 h-5 rounded border-2 p-0 cursor-pointer transition-transform hover:scale-110
                         {editColor === color ? 'border-surface-900 dark:border-surface-50' : 'border-transparent'}"
                  style:background-color={color}
                  onclick={() => editColor = color}
                  title={color}
                ></button>
              {/each}
            </div>
            <div class="flex gap-2">
              <button type="submit" class="btn btn-sm preset-filled-primary-500" disabled={!editName.trim()}>Save</button>
              <button type="button" class="btn btn-sm preset-outlined" onclick={() => editingList = null}>Cancel</button>
            </div>
          </form>
        {:else}
          <div
            class="flex items-center gap-2 p-1.5 rounded-md text-sm group
                   {list.id === listStore.selectedListId ? 'bg-surface-200-800' : 'hover:bg-surface-200-800'}"
          >
            <button
              type="button"
              class="w-3.5 h-3.5 rounded shrink-0 border-none cursor-pointer p-0 hover:scale-115 transition-transform"
              style:background-color={list.color ?? '#6b7280'}
              onclick={() => startEdit(list)}
              title="Edit list"
            ></button>
            <button
              type="button"
              class="flex-1 text-left bg-transparent border-none cursor-pointer text-sm px-1.5 py-1 rounded overflow-hidden text-ellipsis whitespace-nowrap hover:bg-surface-200-800"
              onclick={() => handleNavigation(() => listStore.select(list.id))}
            >
              {list.name}
            </button>
            <button
              type="button"
              class="shrink-0 w-7 h-7 p-0 border-none bg-transparent cursor-pointer rounded flex items-center justify-center
                     text-surface-500 transition-opacity
                     {isMobile ? 'opacity-100 w-11 h-11' : 'opacity-0 group-hover:opacity-100'}
                     hover:bg-error-500 hover:text-white"
              onclick={(e) => handleDeleteClick(list, e)}
              title="Delete list"
            >
              <svg class="w-3.5 h-3.5 {isMobile ? 'w-[18px] h-[18px]' : ''}" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
            </button>
          </div>
        {/if}
      {/each}
    {/if}

    <!-- Account lists sections -->
    {#each [...groupedLists().accountLists.entries()] as [accountId, lists] (accountId)}
      <div class="text-[0.6875rem] font-semibold uppercase tracking-wide text-surface-500 px-3 py-1.5 mt-3">{getAccountName(accountId)}</div>
      {#each lists as list (list.id)}
        {#if editingList?.id === list.id}
          <form class="p-2 bg-surface-200-800 rounded-md mb-1 flex flex-col gap-2" onsubmit={handleEditList} onkeydown={handleKeydown}>
            <input
              type="text"
              class="input text-sm"
              bind:value={editName}
              autofocus
            />
            <div class="flex gap-1.5 flex-wrap">
              {#each COLORS as color}
                <button
                  type="button"
                  class="w-5 h-5 rounded border-2 p-0 cursor-pointer transition-transform hover:scale-110
                         {editColor === color ? 'border-surface-900 dark:border-surface-50' : 'border-transparent'}"
                  style:background-color={color}
                  onclick={() => editColor = color}
                  title={color}
                ></button>
              {/each}
            </div>
            <div class="flex gap-2">
              <button type="submit" class="btn btn-sm preset-filled-primary-500" disabled={!editName.trim()}>Save</button>
              <button type="button" class="btn btn-sm preset-outlined" onclick={() => editingList = null}>Cancel</button>
            </div>
          </form>
        {:else}
          <div
            class="flex items-center gap-2 p-1.5 rounded-md text-sm group
                   {list.id === listStore.selectedListId ? 'bg-surface-200-800' : 'hover:bg-surface-200-800'}"
          >
            <button
              type="button"
              class="w-3.5 h-3.5 rounded shrink-0 border-none cursor-pointer p-0 hover:scale-115 transition-transform"
              style:background-color={list.color ?? '#6b7280'}
              onclick={() => startEdit(list)}
              title="Edit list"
            ></button>
            <button
              type="button"
              class="flex-1 text-left bg-transparent border-none cursor-pointer text-sm px-1.5 py-1 rounded overflow-hidden text-ellipsis whitespace-nowrap hover:bg-surface-200-800"
              onclick={() => handleNavigation(() => listStore.select(list.id))}
            >
              {list.name}
            </button>
            <button
              type="button"
              class="shrink-0 w-7 h-7 p-0 border-none bg-transparent cursor-pointer rounded flex items-center justify-center
                     text-surface-500 transition-opacity
                     {isMobile ? 'opacity-100 w-11 h-11' : 'opacity-0 group-hover:opacity-100'}
                     hover:bg-error-500 hover:text-white"
              onclick={(e) => handleDeleteClick(list, e)}
              title="Delete list"
            >
              <svg class="w-3.5 h-3.5 {isMobile ? 'w-[18px] h-[18px]' : ''}" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
            </button>
          </div>
        {/if}
      {/each}
    {/each}
  </nav>

  {#if listStore.error}
    <p class="px-4 py-2 text-xs text-error-500">{listStore.error}</p>
  {/if}

  <div class="mt-auto p-2 border-t border-surface-300-700">
    {#if syncStore.syncing}
      <div class="flex items-center gap-2 px-3 py-2 text-[0.8125rem] text-primary-500">
        <svg class="w-4 h-4 shrink-0 animate-spin" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
        </svg>
        <span>Syncing...</span>
      </div>
    {/if}
    <button
      class="w-full flex items-center gap-3 px-3 py-2.5 rounded-md text-sm text-left text-surface-500 transition-colors
             {showSettings ? 'bg-surface-200-800 text-surface-900 dark:text-surface-50' : 'hover:bg-surface-200-800 hover:text-surface-900 dark:hover:text-surface-50'}"
      onclick={() => handleNavigation(() => onToggleSettings?.())}
    >
      <svg class="w-[18px] h-[18px] shrink-0" viewBox="0 0 24 24" fill="currentColor">
        <path d="M19.14 12.94c.04-.31.06-.63.06-.94 0-.31-.02-.63-.06-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/>
      </svg>
      <span>Settings</span>
    </button>
  </div>
</aside>

{#if listToDelete}
  <DeleteListModal list={listToDelete} onClose={() => listToDelete = null} />
{/if}
