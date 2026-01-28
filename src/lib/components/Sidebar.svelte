<script lang="ts">
  import type { TaskList } from '$lib/types';
  import { listStore } from '$lib/stores/lists.svelte';
  import { accountStore } from '$lib/stores/accounts.svelte';
  import DeleteListModal from './DeleteListModal.svelte';
  import ListFormModal from './ListFormModal.svelte';
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

  // Modal state
  let showListForm = $state(false);
  let editingList = $state<TaskList | null>(null);
  let listToDelete = $state<TaskList | null>(null);

  // Close sidebar when a navigation item is selected on mobile
  function handleNavigation(action: () => void) {
    action();
    if (isMobile && onClose) {
      onClose();
    }
  }

  function handleAddList() {
    editingList = null;
    showListForm = true;
  }

  function handleEditList(list: TaskList) {
    editingList = list;
    showListForm = true;
  }

  function handleCloseListForm() {
    showListForm = false;
    editingList = null;
  }

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
  on:click={handleAddList}
/>

<!-- Local lists -->
{#if groupedLists().localLists.length > 0}
  <SideNavMenu text="Local" expanded>
    {#each groupedLists().localLists as list (list.id)}
      <SideNavMenuItem
        text={list.name}
        isSelected={list.id === listStore.selectedListId}
        on:click={() => handleNavigation(() => listStore.select(list.id))}
      >
        <span slot="icon" class="list-color" style:background-color={list.color ?? '#6b7280'}></span>
      </SideNavMenuItem>
    {/each}
  </SideNavMenu>
{/if}

<!-- Account lists -->
{#each [...groupedLists().accountLists.entries()] as [accountId, lists] (accountId)}
  <SideNavMenu text={getAccountName(accountId)} expanded>
    {#each lists as list (list.id)}
      <SideNavMenuItem
        text={list.name}
        isSelected={list.id === listStore.selectedListId}
        on:click={() => handleNavigation(() => listStore.select(list.id))}
      >
        <span slot="icon" class="list-color" style:background-color={list.color ?? '#6b7280'}></span>
      </SideNavMenuItem>
    {/each}
  </SideNavMenu>
{/each}

{#if listStore.error}
  <p class="error">{listStore.error}</p>
{/if}

<!-- Modals -->
{#if showListForm}
  <ListFormModal list={editingList} onClose={handleCloseListForm} />
{/if}

{#if listToDelete}
  <DeleteListModal list={listToDelete} onClose={() => listToDelete = null} />
{/if}

<style>
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
