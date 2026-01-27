<script lang="ts">
  import type { TaskList } from '$lib/types';
  import { listStore } from '$lib/stores/lists.svelte';
  import { accountStore } from '$lib/stores/accounts.svelte';
  import DeleteListModal from './DeleteListModal.svelte';
  import {
    SideNavLink,
    SideNavMenu,
    SideNavMenuItem,
    SideNavDivider,
  } from 'carbon-components-svelte';
  import { Time, Warning, Add } from 'carbon-icons-svelte';

  interface Props {
    showSettings?: boolean;
    onToggleSettings?: () => void;
    isMobile?: boolean;
    onClose?: () => void;
  }

  let { showSettings = false, onToggleSettings, isMobile = false, onClose }: Props = $props();

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

<!-- Special Views -->
<SideNavLink
  icon={Time}
  text="Today"
  isSelected={listStore.selectedSpecialView === 'today'}
  on:click={() => handleNavigation(() => listStore.selectSpecial('today'))}
/>
<SideNavLink
  icon={Warning}
  text="Overdue"
  isSelected={listStore.selectedSpecialView === 'overdue'}
  on:click={() => handleNavigation(() => listStore.selectSpecial('overdue'))}
/>

<SideNavDivider />

<!-- Add List Button -->
<SideNavLink
  icon={Add}
  text="Add List"
  on:click={() => isAdding = true}
/>

{#if isAdding}
  <div class="list-form-container">
    <form class="list-form" onsubmit={handleAddList} onkeydown={handleKeydown}>
      <input
        type="text"
        bind:value={newListName}
        placeholder="List name..."
        autofocus
        class="bx--text-input"
      />
      {#if accountStore.accounts.length > 0}
        <select bind:value={newListAccountId} class="bx--select-input">
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
        <button type="submit" class="bx--btn bx--btn--primary bx--btn--sm" disabled={!newListName.trim()}>Add</button>
        <button type="button" class="bx--btn bx--btn--secondary bx--btn--sm" onclick={() => { isAdding = false; newListName = ''; newListColor = COLORS[0]; newListAccountId = null; }}>Cancel</button>
      </div>
    </form>
  </div>
{/if}

<!-- Local lists -->
{#if groupedLists().localLists.length > 0}
  <SideNavMenu text="Local" expanded>
    {#each groupedLists().localLists as list (list.id)}
      {#if editingList?.id === list.id}
        <div class="list-form-container">
          <form class="list-form" onsubmit={handleEditList} onkeydown={handleKeydown}>
            <input
              type="text"
              bind:value={editName}
              autofocus
              class="bx--text-input"
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
              <button type="submit" class="bx--btn bx--btn--primary bx--btn--sm" disabled={!editName.trim()}>Save</button>
              <button type="button" class="bx--btn bx--btn--secondary bx--btn--sm" onclick={() => editingList = null}>Cancel</button>
            </div>
          </form>
        </div>
      {:else}
        <SideNavMenuItem
          text={list.name}
          isSelected={list.id === listStore.selectedListId}
          on:click={() => handleNavigation(() => listStore.select(list.id))}
        >
          <span slot="icon" class="list-color" style:background-color={list.color ?? '#6b7280'}></span>
        </SideNavMenuItem>
      {/if}
    {/each}
  </SideNavMenu>
{/if}

<!-- Account lists -->
{#each [...groupedLists().accountLists.entries()] as [accountId, lists] (accountId)}
  <SideNavMenu text={getAccountName(accountId)} expanded>
    {#each lists as list (list.id)}
      {#if editingList?.id === list.id}
        <div class="list-form-container">
          <form class="list-form" onsubmit={handleEditList} onkeydown={handleKeydown}>
            <input
              type="text"
              bind:value={editName}
              autofocus
              class="bx--text-input"
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
              <button type="submit" class="bx--btn bx--btn--primary bx--btn--sm" disabled={!editName.trim()}>Save</button>
              <button type="button" class="bx--btn bx--btn--secondary bx--btn--sm" onclick={() => editingList = null}>Cancel</button>
            </div>
          </form>
        </div>
      {:else}
        <SideNavMenuItem
          text={list.name}
          isSelected={list.id === listStore.selectedListId}
          on:click={() => handleNavigation(() => listStore.select(list.id))}
        >
          <span slot="icon" class="list-color" style:background-color={list.color ?? '#6b7280'}></span>
        </SideNavMenuItem>
      {/if}
    {/each}
  </SideNavMenu>
{/each}

{#if listStore.error}
  <p class="error">{listStore.error}</p>
{/if}

{#if listToDelete}
  <DeleteListModal list={listToDelete} onClose={() => listToDelete = null} />
{/if}

<style>
  .list-form-container {
    padding: 0.5rem 1rem;
  }

  .list-form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
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
    border-color: var(--cds-text-primary, var(--text-primary));
  }

  .form-buttons {
    display: flex;
    gap: 0.5rem;
  }

  .list-color {
    width: 12px;
    height: 12px;
    border-radius: 2px;
    display: inline-block;
  }

  .error {
    padding: 0.5rem 1rem;
    color: var(--cds-danger, var(--error-color));
    font-size: 0.75rem;
  }
</style>
