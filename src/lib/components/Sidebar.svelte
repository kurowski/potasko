<script lang="ts">
  import type { TaskList } from '$lib/types';
  import { listStore } from '$lib/stores/lists.svelte';
  import { accountStore } from '$lib/stores/accounts.svelte';
  import { syncStore } from '$lib/stores/sync.svelte';
  import { preferencesStore } from '$lib/stores/preferences.svelte';
  import DeleteListModal from './DeleteListModal.svelte';
  import ListItemMenu from './ListItemMenu.svelte';
  import { longpress } from '$lib/actions/longpress';
  import List, { Item, Text, Graphic, Separator, Subheader } from '@smui/list';
  import IconButton from '@smui/icon-button';
  import Textfield from '@smui/textfield';
  import Select, { Option } from '@smui/select';
  import Button from '@smui/button';
  import CircularProgress from '@smui/circular-progress';

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
  let newListAccountId = $state<number | ''>('');
  let isAdding = $state(false);

  // Edit list state
  let editingList = $state<TaskList | null>(null);
  let editName = $state('');
  let editColor = $state('');

  // Delete modal state
  let listToDelete = $state<TaskList | null>(null);

  // Context menu state
  let listItemMenu: ListItemMenu;

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
        account_id: newListAccountId === '' ? null : newListAccountId,
      });
      newListName = '';
      newListColor = COLORS[0];
      newListAccountId = '';
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
      newListAccountId = '';
      editingList = null;
    }
  }

  function openListMenu(e: Event | PointerEvent, list: TaskList) {
    e.preventDefault();
    e.stopPropagation();
    const target = (e.currentTarget ?? e.target) as HTMLElement;
    listItemMenu.openMenu(target, list);
  }

  function handleMenuEdit(list: TaskList) {
    startEdit(list);
  }

  function handleMenuDelete(list: TaskList) {
    listToDelete = list;
  }
</script>

<aside class="sidebar" class:mobile={isMobile} class:open={isOpen}>
  {#if isMobile}
    <div class="mobile-header">
      <h2>Menu</h2>
      <IconButton onclick={onClose}>
        <span class="material-icons">close</span>
      </IconButton>
    </div>
  {/if}

  <div class="section-header">
    <span class="section-title">Views</span>
  </div>

  <List dense>
    <Item
      activated={listStore.selectedSpecialView === 'today'}
      onclick={() => handleNavigation(() => listStore.selectSpecial('today'))}
    >
      <Graphic>
        <span class="material-icons">today</span>
      </Graphic>
      <Text>Today</Text>
    </Item>
    <Item
      activated={listStore.selectedSpecialView === 'overdue'}
      onclick={() => handleNavigation(() => listStore.selectSpecial('overdue'))}
    >
      <Graphic>
        <span class="material-icons">warning</span>
      </Graphic>
      <Text>Overdue</Text>
    </Item>
  </List>

  <Separator />

  <div class="section-header">
    <span class="section-title">Lists</span>
    <IconButton onclick={() => isAdding = true} title="Add list">
      <span class="material-icons">add</span>
    </IconButton>
  </div>

  {#if isAdding}
    <form class="list-form" onsubmit={handleAddList} onkeydown={handleKeydown}>
      <Textfield
        variant="outlined"
        bind:value={newListName}
        label="List name"
        style="width: 100%;"
      />
      {#if accountStore.accounts.length > 0}
        <Select variant="outlined" bind:value={newListAccountId} label="Account" style="width: 100%;">
          {#if preferencesStore.showLocalAccounts}
            <Option value="">Local only</Option>
          {/if}
          {#each accountStore.accounts as account (account.id)}
            <Option value={account.id}>{account.name}</Option>
          {/each}
        </Select>
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
        <Button variant="raised" type="submit" disabled={!newListName.trim()}>Add</Button>
        <Button variant="text" onclick={() => { isAdding = false; newListName = ''; newListColor = COLORS[0]; newListAccountId = ''; }}>Cancel</Button>
      </div>
    </form>
  {/if}

  <div class="list-nav">
    <!-- Local lists section -->
    {#if preferencesStore.showLocalAccounts && groupedLists().localLists.length > 0}
      <Subheader>Local</Subheader>
      <List dense>
        {#each groupedLists().localLists as list (list.id)}
          {#if editingList?.id === list.id}
            <form class="list-form inline" onsubmit={handleEditList} onkeydown={handleKeydown}>
              <Textfield
                variant="outlined"
                bind:value={editName}
                style="width: 100%;"
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
                <Button variant="raised" type="submit" disabled={!editName.trim()}>Save</Button>
                <Button variant="text" onclick={() => editingList = null}>Cancel</Button>
              </div>
            </form>
          {:else}
            <Item
              activated={list.id === listStore.selectedListId}
              onclick={() => handleNavigation(() => listStore.select(list.id))}
              use={[[longpress, { onLongpress: (e: PointerEvent) => openListMenu(e, list) }]]}
              oncontextmenu={(e: Event) => openListMenu(e, list)}
            >
              <Graphic>
                <span
                  class="list-color-indicator"
                  style:background-color={list.color ?? '#6b7280'}
                ></span>
              </Graphic>
              <Text>{list.name}</Text>
              {#if !isMobile}
                <span class="overflow-wrapper">
                  <IconButton
                    onclick={(e: Event) => openListMenu(e, list)}
                    title="More options"
                    size="button"
                  >
                    <span class="material-icons overflow-icon">more_vert</span>
                  </IconButton>
                </span>
              {/if}
            </Item>
          {/if}
        {/each}
      </List>
    {/if}

    <!-- Account lists sections -->
    {#each [...groupedLists().accountLists.entries()] as [accountId, lists] (accountId)}
      <Subheader>{getAccountName(accountId)}</Subheader>
      <List dense>
        {#each lists as list (list.id)}
          {#if editingList?.id === list.id}
            <form class="list-form inline" onsubmit={handleEditList} onkeydown={handleKeydown}>
              <Textfield
                variant="outlined"
                bind:value={editName}
                style="width: 100%;"
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
                <Button variant="raised" type="submit" disabled={!editName.trim()}>Save</Button>
                <Button variant="text" onclick={() => editingList = null}>Cancel</Button>
              </div>
            </form>
          {:else}
            <Item
              activated={list.id === listStore.selectedListId}
              onclick={() => handleNavigation(() => listStore.select(list.id))}
              use={[[longpress, { onLongpress: (e: PointerEvent) => openListMenu(e, list) }]]}
              oncontextmenu={(e: Event) => openListMenu(e, list)}
            >
              <Graphic>
                <span
                  class="list-color-indicator"
                  style:background-color={list.color ?? '#6b7280'}
                ></span>
              </Graphic>
              <Text>{list.name}</Text>
              {#if !isMobile}
                <span class="overflow-wrapper">
                  <IconButton
                    onclick={(e: Event) => openListMenu(e, list)}
                    title="More options"
                    size="button"
                  >
                    <span class="material-icons overflow-icon">more_vert</span>
                  </IconButton>
                </span>
              {/if}
            </Item>
          {/if}
        {/each}
      </List>
    {/each}
  </div>

  {#if listStore.error}
    <p class="error">{listStore.error}</p>
  {/if}

  <div class="sidebar-footer">
    {#if syncStore.syncing}
      <div class="sync-indicator">
        <CircularProgress style="width: 16px; height: 16px;" indeterminate />
        <span>Syncing...</span>
      </div>
    {/if}
    <List dense>
      <Item
        activated={showSettings}
        onclick={() => handleNavigation(() => onToggleSettings?.())}
      >
        <Graphic>
          <span class="material-icons">settings</span>
        </Graphic>
        <Text>Settings</Text>
      </Item>
    </List>
  </div>
</aside>

{#if listToDelete}
  <DeleteListModal list={listToDelete} onClose={() => listToDelete = null} />
{/if}

<ListItemMenu
  bind:this={listItemMenu}
  onEdit={handleMenuEdit}
  onDelete={handleMenuDelete}
/>

<style>
  .sidebar {
    width: 260px;
    background: var(--mdc-theme-surface, #fff);
    border-right: 1px solid var(--app-color-border);
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
    z-index: var(--app-z-drawer);
    transform: translateX(-100%);
    transition: transform var(--app-transition-slow);
    border-right: none;
    box-shadow: var(--app-shadow-drawer);
  }

  .sidebar.mobile.open {
    transform: translateX(0);
  }

  .mobile-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--app-spacing-2) var(--app-spacing-2) var(--app-spacing-2) var(--app-spacing-4);
    padding-top: calc(var(--app-spacing-2) + env(safe-area-inset-top, 0));
    border-bottom: 1px solid var(--app-color-border);
  }

  .mobile-header h2 {
    margin: 0;
    font-size: var(--app-font-size-lg);
    font-weight: 500;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--app-spacing-4) var(--app-spacing-4) var(--app-spacing-2);
  }

  .section-title {
    font-size: var(--app-font-size-2xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .list-form {
    padding: var(--app-spacing-3) var(--app-spacing-4);
    display: flex;
    flex-direction: column;
    gap: var(--app-spacing-3);
    border-bottom: 1px solid var(--app-color-border);
  }

  .list-form.inline {
    padding: var(--app-spacing-3);
    border-bottom: none;
    background: var(--app-color-surface-hover);
    border-radius: var(--app-radius-md);
    margin: var(--app-spacing-1) var(--app-spacing-2);
  }

  .color-picker {
    display: flex;
    gap: 0.375rem;
    flex-wrap: wrap;
  }

  .color-option {
    width: 24px;
    height: 24px;
    border: 2px solid transparent;
    border-radius: var(--app-radius-sm);
    cursor: pointer;
    padding: 0;
    transition: transform var(--app-transition-fast);
  }

  .color-option:hover {
    transform: scale(1.1);
  }

  .color-option.selected {
    border-color: var(--mdc-theme-on-surface, #000);
  }

  .form-buttons {
    display: flex;
    gap: var(--app-spacing-2);
  }

  .list-nav {
    flex: 1;
    overflow-y: auto;
  }

  .list-color-indicator {
    width: 14px;
    height: 14px;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .overflow-wrapper {
    opacity: 0;
    transition: opacity var(--app-transition-normal);
  }

  :global(.mdc-deprecated-list-item:hover) .overflow-wrapper {
    opacity: 1;
  }

  .overflow-icon {
    font-size: 18px;
  }

  .error {
    padding: var(--app-spacing-2) var(--app-spacing-4);
    color: var(--mdc-theme-error, #dc2626);
    font-size: var(--app-font-size-xs);
  }

  .sidebar-footer {
    margin-top: auto;
    border-top: 1px solid var(--app-color-border);
  }

  .sync-indicator {
    display: flex;
    align-items: center;
    gap: var(--app-spacing-2);
    padding: var(--app-spacing-3) var(--app-spacing-4);
    font-size: 0.8125rem;
    color: var(--mdc-theme-primary, #3b82f6);
  }
</style>
