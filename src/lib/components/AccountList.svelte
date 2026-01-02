<script lang="ts">
  import { accountStore } from '$lib/stores/accounts.svelte';
  import AccountForm from './AccountForm.svelte';
  import type { Account } from '$lib/types';

  // UI state
  let showAddForm = $state(false);
  let editingAccount = $state<Account | null>(null);

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
</script>

<div class="account-list">
  <div class="header">
    <h2>CalDAV Accounts</h2>
    <button class="add-btn" onclick={handleAddClick}>
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
      </svg>
      Add Account
    </button>
  </div>

  {#if accountStore.loading}
    <p class="loading">Loading accounts...</p>
  {:else if accountStore.accounts.length === 0 && !showAddForm}
    <div class="empty-state">
      <svg viewBox="0 0 24 24" fill="currentColor" class="empty-icon">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
      </svg>
      <p>No CalDAV accounts configured</p>
      <p class="hint">Add an account to sync tasks with your CalDAV server</p>
    </div>
  {:else}
    <ul class="accounts">
      {#each accountStore.accounts as account (account.id)}
        <li class="account-item">
          <div class="account-info">
            <span class="account-name">{account.name}</span>
            <span class="account-server">{account.server_url}</span>
            <span class="account-user">{account.username}</span>
          </div>
          <div class="account-actions">
            <button
              class="icon-btn"
              onclick={() => handleEditClick(account)}
              title="Edit account"
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34a.9959.9959 0 0 0-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
              </svg>
            </button>
            <button
              class="icon-btn danger"
              onclick={() => handleDeleteClick(account)}
              title="Delete account"
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  {#if showAddForm}
    <AccountForm onClose={handleFormClose} onSaved={handleFormSaved} />
  {/if}

  {#if editingAccount}
    <AccountForm account={editingAccount} onClose={handleFormClose} onSaved={handleFormSaved} />
  {/if}

  {#if accountStore.error && !showAddForm && !editingAccount}
    <p class="error">{accountStore.error}</p>
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

  .add-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: var(--accent-color);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .add-btn svg {
    width: 18px;
    height: 18px;
  }

  .add-btn:hover {
    opacity: 0.9;
  }

  .loading {
    color: var(--text-secondary);
    text-align: center;
    padding: 2rem;
  }

  .empty-state {
    text-align: center;
    padding: 3rem 2rem;
    color: var(--text-secondary);
  }

  .empty-icon {
    width: 48px;
    height: 48px;
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
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .account-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: var(--bg-secondary);
    border-radius: 8px;
    border: 1px solid var(--border-color);
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
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .account-user {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    opacity: 0.8;
  }

  .account-actions {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .icon-btn {
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
  }

  .icon-btn svg {
    width: 18px;
    height: 18px;
  }

  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .icon-btn.danger:hover {
    background: var(--priority-high-bg);
    color: var(--priority-high-text);
  }

  .error {
    color: var(--error-color);
    font-size: 0.875rem;
    padding: 0.75rem;
    background: var(--priority-high-bg);
    border-radius: 6px;
    margin: 0;
  }
</style>
