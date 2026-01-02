<script lang="ts">
  import { accountStore } from '$lib/stores/accounts.svelte';
  import type { Account, CalendarInfo } from '$lib/types';

  interface Props {
    account?: Account | null;
    onClose?: () => void;
    onSaved?: () => void;
  }

  let { account = null, onClose, onSaved }: Props = $props();

  // Form state
  let name = $state(account?.name ?? '');
  let serverUrl = $state(account?.server_url ?? '');
  let username = $state(account?.username ?? '');
  let password = $state(account?.password ?? '');
  let saving = $state(false);

  const isEditing = $derived(account !== null);

  // Discovered data from test
  let principalUrl = $state<string | null>(account?.principal_url ?? null);
  let calendarHomeUrl = $state<string | null>(account?.calendar_home_url ?? null);
  let discoveredCalendars = $state<CalendarInfo[]>([]);
  let connectionTested = $state(false);

  async function handleTestConnection() {
    if (!serverUrl.trim() || !username.trim() || !password.trim()) return;

    try {
      const result = await accountStore.testConnection(serverUrl.trim(), username.trim(), password.trim());
      if (result.success) {
        principalUrl = result.principal_url;
        calendarHomeUrl = result.calendar_home_url;
        discoveredCalendars = result.calendars;
        connectionTested = true;
      }
    } catch {
      // Error handled by store
      connectionTested = false;
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!name.trim() || !serverUrl.trim() || !username.trim() || !password.trim()) return;

    saving = true;
    try {
      const data = {
        name: name.trim(),
        server_url: serverUrl.trim(),
        username: username.trim(),
        password: password.trim(),
        principal_url: principalUrl,
        calendar_home_url: calendarHomeUrl,
      };

      if (isEditing && account) {
        await accountStore.update(account.id, data);
      } else {
        await accountStore.create(data);
      }

      onSaved?.();
      onClose?.();
    } catch {
      // Error handled by store
    } finally {
      saving = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose?.();
    }
  }

  // Reset test results when connection params change
  $effect(() => {
    // Track these to trigger reset
    serverUrl; username; password;
    // Reset if params changed after a successful test
    if (connectionTested) {
      connectionTested = false;
      principalUrl = null;
      calendarHomeUrl = null;
      discoveredCalendars = [];
    }
  });
</script>

<form class="account-form" onsubmit={handleSubmit} onkeydown={handleKeydown}>
  <h3>{isEditing ? 'Edit Account' : 'Add CalDAV Account'}</h3>

  <label class="form-field">
    <span>Account Name</span>
    <input
      type="text"
      bind:value={name}
      placeholder="e.g., Work, Personal"
      disabled={saving}
    />
  </label>

  <label class="form-field">
    <span>Server URL</span>
    <input
      type="url"
      bind:value={serverUrl}
      placeholder="https://caldav.example.com"
      disabled={saving}
    />
  </label>

  <label class="form-field">
    <span>Username</span>
    <input
      type="text"
      bind:value={username}
      placeholder="Username"
      disabled={saving}
    />
  </label>

  <label class="form-field">
    <span>Password</span>
    <input
      type="password"
      bind:value={password}
      placeholder="Password"
      disabled={saving}
    />
  </label>

  <div class="test-section">
    <button
      type="button"
      class="test-btn"
      onclick={handleTestConnection}
      disabled={!serverUrl.trim() || !username.trim() || !password.trim() || accountStore.testing || saving}
    >
      {accountStore.testing ? 'Testing...' : 'Test Connection'}
    </button>

    {#if accountStore.testResult}
      {#if accountStore.testResult.success}
        <div class="test-result success">
          <svg class="icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
          </svg>
          <span>Connection successful!</span>
        </div>
        {#if discoveredCalendars.length > 0}
          <div class="calendars">
            <span class="calendars-label">Discovered calendars with tasks:</span>
            <ul>
              {#each discoveredCalendars.filter(c => c.supports_vtodo) as cal}
                <li>
                  {#if cal.color}
                    <span class="cal-color" style:background-color={cal.color}></span>
                  {/if}
                  {cal.display_name || cal.href}
                </li>
              {/each}
              {#if discoveredCalendars.filter(c => c.supports_vtodo).length === 0}
                <li class="no-calendars">No calendars with task support found</li>
              {/if}
            </ul>
          </div>
        {/if}
      {:else}
        <div class="test-result error">
          <svg class="icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
          </svg>
          <span>{accountStore.testResult.error || 'Connection failed'}</span>
        </div>
      {/if}
    {/if}
  </div>

  {#if accountStore.error && !accountStore.testResult}
    <p class="error">{accountStore.error}</p>
  {/if}

  <div class="form-actions">
    <button type="button" onclick={onClose} disabled={saving}>Cancel</button>
    <button
      type="submit"
      class="primary"
      disabled={!name.trim() || !serverUrl.trim() || !username.trim() || !password.trim() || saving}
    >
      {saving ? 'Saving...' : isEditing ? 'Save' : 'Add Account'}
    </button>
  </div>
</form>

<style>
  .account-form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1.5rem;
    background: var(--bg-secondary);
    border-radius: 8px;
  }

  h3 {
    margin: 0 0 0.5rem;
    font-size: 1.125rem;
    font-weight: 600;
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .form-field span {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .form-field input {
    padding: 0.625rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    font-size: 0.9375rem;
    background: var(--bg-primary);
  }

  .form-field input:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .test-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding-top: 0.5rem;
  }

  .test-btn {
    align-self: flex-start;
    padding: 0.5rem 1rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--bg-primary);
    cursor: pointer;
    font-size: 0.875rem;
  }

  .test-btn:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .test-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .test-result {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    border-radius: 6px;
    font-size: 0.875rem;
  }

  .test-result.success {
    background: var(--priority-low-bg);
    color: var(--priority-low-text);
  }

  .test-result.error {
    background: var(--priority-high-bg);
    color: var(--priority-high-text);
  }

  .test-result .icon {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
  }

  .calendars {
    padding: 0.75rem;
    background: var(--bg-primary);
    border-radius: 6px;
    font-size: 0.875rem;
  }

  .calendars-label {
    display: block;
    margin-bottom: 0.5rem;
    color: var(--text-secondary);
  }

  .calendars ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .calendars li {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .cal-color {
    width: 12px;
    height: 12px;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .no-calendars {
    color: var(--text-secondary);
    font-style: italic;
  }

  .error {
    color: var(--error-color);
    font-size: 0.875rem;
    margin: 0;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding-top: 0.5rem;
  }

  .form-actions button {
    padding: 0.625rem 1.25rem;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .form-actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .form-actions button.primary {
    background: var(--accent-color);
    color: white;
  }

  .form-actions button:not(.primary) {
    background: transparent;
    border: 1px solid var(--border-color);
  }

  .form-actions button:not(.primary):hover:not(:disabled) {
    background: var(--bg-hover);
  }
</style>
