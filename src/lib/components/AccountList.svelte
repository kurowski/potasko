<script lang="ts">
  import { accountStore } from '$lib/stores/accounts.svelte';
  import { syncStore } from '$lib/stores/sync.svelte';
  import { listStore } from '$lib/stores/lists.svelte';
  import AccountForm from './AccountForm.svelte';
  import SyncLogModal from './SyncLogModal.svelte';
  import type { Account } from '$lib/types';
  import {
    Button,
    Tile,
    InlineLoading,
    InlineNotification,
    TooltipDefinition,
  } from 'carbon-components-svelte';
  import { Add, Renew, Edit, TrashCan, Checkmark } from 'carbon-icons-svelte';

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

<div class="account-list">
  <div class="header">
    <h2>CalDAV Accounts</h2>
    <Button kind="primary" size="small" icon={Add} on:click={handleAddClick}>
      Add Account
    </Button>
  </div>

  {#if accountStore.loading}
    <InlineLoading description="Loading accounts..." />
  {:else if accountStore.accounts.length === 0 && !showAddForm}
    <Tile class="empty-state">
      <Checkmark size={48} class="empty-icon" />
      <p>No CalDAV accounts configured</p>
      <p class="hint">Add an account to sync tasks with your CalDAV server</p>
    </Tile>
  {:else}
    <div class="accounts">
      {#each accountStore.accounts as account (account.id)}
        <Tile class="account-item">
          <div class="account-info">
            <span class="account-name">{account.name}</span>
            <span class="account-server">{account.server_url}</span>
            <span class="account-user">{account.username}</span>
          </div>
          <div class="account-actions">
            <Button
              kind="ghost"
              size="small"
              icon={Renew}
              iconDescription="Sync account"
              on:click={() => handleSyncClick(account)}
              disabled={syncStore.syncing}
              class={syncStore.syncing && syncStore.syncingAccountId === account.id ? 'spinning' : ''}
            />
            <Button
              kind="ghost"
              size="small"
              icon={Edit}
              iconDescription="Edit account"
              on:click={() => handleEditClick(account)}
            />
            <Button
              kind="danger-ghost"
              size="small"
              icon={TrashCan}
              iconDescription="Delete account"
              on:click={() => handleDeleteClick(account)}
            />
          </div>
        </Tile>
      {/each}
    </div>

    {#if syncStore.lastResult}
      <InlineNotification
        kind={syncStore.lastResult.success ? 'success' : 'error'}
        title={syncStore.lastResult.success ? 'Sync complete' : 'Sync failed'}
        subtitle={syncStore.lastResult.success ? formatSyncResult() : (syncStore.lastResult.error || 'Unknown error')}
        hideCloseButton
      />
    {/if}

    <Button kind="tertiary" size="small" on:click={() => showSyncLog = true} class="view-log-btn">
      View Sync Log
    </Button>
  {/if}

  {#if showAddForm}
    <AccountForm onClose={handleFormClose} onSaved={handleFormSaved} />
  {/if}

  {#if editingAccount}
    <AccountForm account={editingAccount} onClose={handleFormClose} onSaved={handleFormSaved} />
  {/if}

  {#if accountStore.error && !showAddForm && !editingAccount}
    <InlineNotification
      kind="error"
      title="Error"
      subtitle={accountStore.error}
      hideCloseButton
    />
  {/if}

  {#if showSyncLog}
    <SyncLogModal onClose={() => showSyncLog = false} />
  {/if}
</div>

<style>
  .account-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .header h2 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  :global(.empty-state) {
    text-align: center;
    padding: 3rem 2rem !important;
    color: var(--cds-text-secondary, var(--text-secondary));
  }

  :global(.empty-icon) {
    opacity: 0.5;
    margin-bottom: 1rem;
  }

  .empty-state p {
    margin: 0.25rem 0;
  }

  .empty-state .hint {
    font-size: 0.875rem;
    opacity: 0.8;
  }

  .accounts {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  :global(.account-item) {
    display: flex !important;
    justify-content: space-between;
    align-items: center;
  }

  .account-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
  }

  .account-name {
    font-weight: 500;
    font-size: 1rem;
  }

  .account-server {
    font-size: 0.875rem;
    color: var(--cds-text-secondary, var(--text-secondary));
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .account-user {
    font-size: 0.8125rem;
    color: var(--cds-text-secondary, var(--text-secondary));
    opacity: 0.8;
  }

  .account-actions {
    display: flex;
    gap: 0.25rem;
    flex-shrink: 0;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  :global(.view-log-btn) {
    width: 100%;
    justify-content: center;
  }
</style>
