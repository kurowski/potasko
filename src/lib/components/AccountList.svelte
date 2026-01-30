<script lang="ts">
  import { accountStore } from '$lib/stores/accounts.svelte';
  import { syncStore } from '$lib/stores/sync.svelte';
  import { listStore } from '$lib/stores/lists.svelte';
  import AccountForm from './AccountForm.svelte';
  import SyncLogModal from './SyncLogModal.svelte';
  import type { Account } from '$lib/types';
  import Button from '@smui/button';
  import IconButton from '@smui/icon-button';
  import CircularProgress from '@smui/circular-progress';
  import Card from '@smui/card';

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
    <Button variant="raised" onclick={handleAddClick}>
      <span class="material-icons" style="font-size: 18px; margin-right: 8px;">add</span>
      Add Account
    </Button>
  </div>

  {#if accountStore.loading}
    <div class="loading">
      <CircularProgress indeterminate />
      <p>Loading accounts...</p>
    </div>
  {:else if accountStore.accounts.length === 0 && !showAddForm}
    <div class="empty-state">
      <span class="material-icons empty-icon">cloud_off</span>
      <p>No CalDAV accounts configured</p>
      <p class="hint">Add an account to sync tasks with your CalDAV server</p>
    </div>
  {:else}
    <div class="accounts">
      {#each accountStore.accounts as account (account.id)}
        <Card class="account-card">
          <div class="account-item">
            <div class="account-info">
              <span class="account-name">{account.name}</span>
              <span class="account-server">{account.server_url}</span>
              <span class="account-user">{account.username}</span>
            </div>
            <div class="account-actions">
              <IconButton
                onclick={() => handleSyncClick(account)}
                disabled={syncStore.syncing}
                title="Sync account"
                aria-label="Sync account"
              >
                {#if syncStore.syncing && syncStore.syncingAccountId === account.id}
                  <CircularProgress style="width: 24px; height: 24px;" indeterminate />
                {:else}
                  <span class="material-icons">sync</span>
                {/if}
              </IconButton>
              <IconButton
                onclick={() => handleEditClick(account)}
                title="Edit account"
              >
                <span class="material-icons">edit</span>
              </IconButton>
              <IconButton
                onclick={() => handleDeleteClick(account)}
                title="Delete account"
                class="danger"
              >
                <span class="material-icons">delete</span>
              </IconButton>
            </div>
          </div>
        </Card>
      {/each}
    </div>

    {#if syncStore.lastResult}
      <div class="sync-result" class:success={syncStore.lastResult.success} class:error={!syncStore.lastResult.success}>
        {#if syncStore.lastResult.success}
          <span class="material-icons">check_circle</span>
          <span>{formatSyncResult()}</span>
        {:else}
          <span class="material-icons">error</span>
          <span>{syncStore.lastResult.error || 'Sync failed'}</span>
        {/if}
      </div>
    {/if}

    <Button variant="outlined" onclick={() => showSyncLog = true} style="width: 100%; margin-top: 1rem;">
      <span class="material-icons" style="font-size: 18px; margin-right: 8px;">history</span>
      View Sync Log
    </Button>
  {/if}

  {#if showAddForm}
    <Card class="form-card">
      <AccountForm onClose={handleFormClose} onSaved={handleFormSaved} />
    </Card>
  {/if}

  {#if editingAccount}
    <Card class="form-card">
      <AccountForm account={editingAccount} onClose={handleFormClose} onSaved={handleFormSaved} />
    </Card>
  {/if}

  {#if accountStore.error && !showAddForm && !editingAccount}
    <div class="error-message">
      <span class="material-icons">error</span>
      {accountStore.error}
    </div>
  {/if}

  {#if showSyncLog}
    <SyncLogModal onClose={() => showSyncLog = false} />
  {/if}
</div>

<style>
  .account-list {
    display: flex;
    flex-direction: column;
    gap: var(--app-spacing-4);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--app-spacing-4);
  }

  .header h2 {
    margin: 0;
    font-size: var(--app-font-size-lg);
    font-weight: 500;
  }

  .loading {
    padding: 2rem;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--app-spacing-4);
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .loading p {
    margin: 0;
  }

  .empty-state {
    text-align: center;
    padding: 3rem 2rem;
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .empty-icon {
    font-size: 48px;
    opacity: 0.5;
    margin-bottom: 0.5rem;
  }

  .empty-state p {
    margin: var(--app-spacing-1) 0;
  }

  .empty-state .hint {
    font-size: var(--app-font-size-sm);
    opacity: 0.8;
  }

  .accounts {
    display: flex;
    flex-direction: column;
    gap: var(--app-spacing-3);
  }

  :global(.account-card) {
    padding: 0 !important;
  }

  .account-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--app-spacing-4);
    gap: var(--app-spacing-4);
  }

  .account-info {
    display: flex;
    flex-direction: column;
    gap: var(--app-spacing-1);
    min-width: 0;
  }

  .account-name {
    font-weight: 500;
    font-size: var(--app-font-size-base);
  }

  .account-server {
    font-size: var(--app-font-size-sm);
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .account-user {
    font-size: 0.8125rem;
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
    opacity: 0.8;
  }

  .account-actions {
    display: flex;
    gap: var(--app-spacing-1);
    flex-shrink: 0;
  }

  :global(.danger .material-icons) {
    color: var(--mdc-theme-error, #dc2626);
  }

  .sync-result {
    display: flex;
    align-items: center;
    gap: var(--app-spacing-2);
    padding: var(--app-spacing-3) var(--app-spacing-4);
    border-radius: var(--app-radius-md);
    font-size: var(--app-font-size-sm);
  }

  .sync-result.success {
    background: var(--app-color-status-success-bg);
    color: var(--app-color-status-success-text);
  }

  .sync-result.error {
    background: var(--app-color-status-error-bg);
    color: var(--app-color-status-error-text);
  }

  .sync-result .material-icons {
    font-size: 20px;
  }

  :global(.form-card) {
    margin-top: var(--app-spacing-4);
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: var(--app-spacing-2);
    color: var(--app-color-status-error-text);
    font-size: var(--app-font-size-sm);
    padding: var(--app-spacing-3) var(--app-spacing-4);
    background: var(--app-color-status-error-bg);
    border-radius: var(--app-radius-md);
  }

  .error-message .material-icons {
    font-size: 20px;
  }
</style>
