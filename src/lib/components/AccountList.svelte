<script lang="ts">
  import { accountStore } from '$lib/stores/accounts.svelte';
  import { syncStore } from '$lib/stores/sync.svelte';
  import { listStore } from '$lib/stores/lists.svelte';
  import AccountForm from './AccountForm.svelte';
  import SyncLogModal from './SyncLogModal.svelte';
  import type { Account } from '$lib/types';

  // UI state
  let showAddForm = $state(false);
  let editingAccount = $state<Account | null>(null);
  let showSyncLog = $state(false);

  // Load accounts on mount
  $effect(() => {
    accountStore.load();
  });

  function handleAddClick() {
    editingAccount = null;
    showAddForm = true;
    accountStore.clearTestResult();
  }

  function handleEditClick(account: Account) {
    showAddForm = false;
    editingAccount = account;
    accountStore.clearTestResult();
  }

  async function handleDeleteClick(account: Account) {
    if (confirm(`Delete account "${account.name}"? This will not delete any synced tasks.`)) {
      try {
        await accountStore.remove(account.id);
      } catch {
        // Error handled by store
      }
    }
  }

  function handleFormClose() {
    showAddForm = false;
    editingAccount = null;
    accountStore.clearTestResult();
  }

  function handleFormSaved() {
    // Reload accounts after save
    accountStore.load();
  }

  async function handleSyncClick(account: Account) {
    await syncStore.syncAccount(account.id, () => {
      // Refresh lists after sync (may have imported new calendars)
      listStore.load();
    });
  }

  // Format sync result message
  function formatSyncResult(): string {
    const result = syncStore.lastResult;
    if (!result) return '';

    const parts: string[] = [];

    if (result.calendars_imported > 0) {
      parts.push(`${result.calendars_imported} calendar(s) imported`);
    }

    // Aggregate stats across all lists
    let pushed = 0, pulled = 0;
    for (const lr of result.list_results) {
      pushed += lr.stats.pushed_created + lr.stats.pushed_updated;
      pulled += lr.stats.pulled_created + lr.stats.pulled_updated;
    }

    if (pushed > 0) parts.push(`${pushed} pushed`);
    if (pulled > 0) parts.push(`${pulled} pulled`);

    if (parts.length === 0) {
      return 'No changes';
    }

    return parts.join(', ');
  }
</script>

<div class="flex flex-col gap-4">
  <div class="flex justify-between items-center">
    <h2 class="m-0 text-xl font-semibold">CalDAV Accounts</h2>
    <button class="btn preset-filled-primary-500" onclick={handleAddClick}>
      <svg class="w-[18px] h-[18px]" viewBox="0 0 24 24" fill="currentColor">
        <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
      </svg>
      Add Account
    </button>
  </div>

  {#if accountStore.loading}
    <p class="text-surface-500 text-center py-8">Loading accounts...</p>
  {:else if accountStore.accounts.length === 0 && !showAddForm}
    <div class="text-center py-12 text-surface-500">
      <svg class="w-12 h-12 opacity-50 mx-auto mb-4" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
      </svg>
      <p class="my-1">No CalDAV accounts configured</p>
      <p class="my-1 text-sm opacity-80">Add an account to sync tasks with your CalDAV server</p>
    </div>
  {:else}
    <ul class="list-none m-0 p-0 flex flex-col gap-2">
      {#each accountStore.accounts as account (account.id)}
        <li class="card flex justify-between items-center p-4 bg-surface-100-900 rounded-lg border border-surface-300-700">
          <div class="flex flex-col gap-1 min-w-0">
            <span class="font-medium">{account.name}</span>
            <span class="text-sm text-surface-500 overflow-hidden text-ellipsis whitespace-nowrap">{account.server_url}</span>
            <span class="text-[0.8125rem] text-surface-500 opacity-80">{account.username}</span>
          </div>
          <div class="flex gap-2 shrink-0">
            <button
              class="btn-icon preset-tonal text-primary-500"
              onclick={() => handleSyncClick(account)}
              disabled={syncStore.syncing}
              title="Sync account"
            >
              {#if syncStore.syncing && syncStore.syncingAccountId === account.id}
                <svg class="w-[18px] h-[18px] animate-spin" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
                </svg>
              {:else}
                <svg class="w-[18px] h-[18px]" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
                </svg>
              {/if}
            </button>
            <button
              class="btn-icon preset-tonal"
              onclick={() => handleEditClick(account)}
              title="Edit account"
            >
              <svg class="w-[18px] h-[18px]" viewBox="0 0 24 24" fill="currentColor">
                <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34a.9959.9959 0 0 0-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
              </svg>
            </button>
            <button
              class="btn-icon preset-tonal hover:preset-filled-error-500"
              onclick={() => handleDeleteClick(account)}
              title="Delete account"
            >
              <svg class="w-[18px] h-[18px]" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
            </button>
          </div>
        </li>
      {/each}
    </ul>

    {#if syncStore.lastResult}
      <div class="flex items-center gap-2 p-3 rounded-md text-sm mt-2
                  {syncStore.lastResult.success ? 'bg-success-500/20 text-success-700 dark:text-success-300' : 'bg-error-500/20 text-error-500'}">
        {#if syncStore.lastResult.success}
          <span class="font-bold">&#10003;</span>
          <span>{formatSyncResult()}</span>
        {:else}
          <span class="font-bold">&#10007;</span>
          <span>{syncStore.lastResult.error || 'Sync failed'}</span>
        {/if}
      </div>
    {/if}

    <button class="btn preset-outlined w-full mt-3 text-[0.8125rem]" onclick={() => showSyncLog = true}>
      View Sync Log
    </button>
  {/if}

  {#if showAddForm}
    <AccountForm onClose={handleFormClose} onSaved={handleFormSaved} />
  {/if}

  {#if editingAccount}
    <AccountForm account={editingAccount} onClose={handleFormClose} onSaved={handleFormSaved} />
  {/if}

  {#if accountStore.error && !showAddForm && !editingAccount}
    <p class="m-0 p-3 bg-error-500/20 rounded-md text-sm text-error-500">{accountStore.error}</p>
  {/if}

  {#if showSyncLog}
    <SyncLogModal onClose={() => showSyncLog = false} />
  {/if}
</div>
