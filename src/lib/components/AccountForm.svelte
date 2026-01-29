<script lang="ts">
  import { untrack } from 'svelte';
  import { accountStore } from '$lib/stores/accounts.svelte';
  import { listStore } from '$lib/stores/lists.svelte';
  import type { Account, CalendarInfo } from '$lib/types';
  import Textfield from '@smui/textfield';
  import Button from '@smui/button';
  import List, { Item, Text, Graphic } from '@smui/list';

  interface Props {
    account?: Account | null;
    onClose?: () => void;
    onSaved?: () => void;
  }

  let { account = null, onClose, onSaved }: Props = $props();

  // Form state - initialized from props (untrack captures initial value only)
  let name = $state(untrack(() => account?.name ?? ''));
  let serverUrl = $state(untrack(() => account?.server_url ?? ''));
  let username = $state(untrack(() => account?.username ?? ''));
  let password = $state(untrack(() => account?.password ?? ''));
  let saving = $state(false);

  const isEditing = $derived(account !== null);

  // Discovered data from test
  let principalUrl = $state<string | null>(untrack(() => account?.principal_url ?? null));
  let calendarHomeUrl = $state<string | null>(untrack(() => account?.calendar_home_url ?? null));
  let discoveredCalendars = $state<CalendarInfo[]>([]);

  // Track what connection params were tested to detect changes
  let testedServerUrl = $state<string | null>(null);
  let testedUsername = $state<string | null>(null);
  let testedPassword = $state<string | null>(null);

  // Connection is valid only if params match what was tested
  const connectionValid = $derived(
    principalUrl !== null &&
    calendarHomeUrl !== null &&
    testedServerUrl === serverUrl.trim() &&
    testedUsername === username.trim() &&
    testedPassword === password.trim()
  );

  async function handleTestConnection() {
    if (!serverUrl.trim() || !username.trim() || !password.trim()) return;

    try {
      const result = await accountStore.testConnection(serverUrl.trim(), username.trim(), password.trim());
      if (result.success) {
        principalUrl = result.principal_url;
        calendarHomeUrl = result.calendar_home_url;
        discoveredCalendars = result.calendars;
        // Remember which params were successfully tested
        testedServerUrl = serverUrl.trim();
        testedUsername = username.trim();
        testedPassword = password.trim();
      }
    } catch {
      // Error handled by store
      principalUrl = null;
      calendarHomeUrl = null;
      testedServerUrl = null;
      testedUsername = null;
      testedPassword = null;
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
        // Create account and immediately sync to import calendars
        await accountStore.createAndSync(data);
        // Refresh lists after sync imports calendars
        listStore.load();
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

  // Clear test result display when connection params change (after initial test)
  let hasTestedOnce = $state(false);
  $effect(() => {
    // Track connection params
    serverUrl; username; password;
    // Only clear if we've tested before (avoid clearing on mount)
    if (hasTestedOnce && !connectionValid) {
      discoveredCalendars = [];
      accountStore.clearTestResult();
    }
  });

  // Mark that we've tested when test completes
  $effect(() => {
    if (connectionValid) {
      hasTestedOnce = true;
    }
  });

  const todoCalendars = $derived(discoveredCalendars.filter(c => c.supports_vtodo));
</script>

<svelte:window onkeydown={handleKeydown} />

<form class="account-form" onsubmit={handleSubmit}>
  <h3>{isEditing ? 'Edit Account' : 'Add CalDAV Account'}</h3>

  <Textfield
    variant="outlined"
    bind:value={name}
    label="Account Name"
    disabled={saving}
    style="width: 100%;"
  />

  <Textfield
    variant="outlined"
    type="url"
    bind:value={serverUrl}
    label="Server URL"
    disabled={saving}
    style="width: 100%;"
  />

  <Textfield
    variant="outlined"
    bind:value={username}
    label="Username"
    disabled={saving}
    style="width: 100%;"
  />

  <Textfield
    variant="outlined"
    type="password"
    bind:value={password}
    label="Password"
    disabled={saving}
    style="width: 100%;"
  />

  <div class="test-section">
    <Button
      variant="outlined"
      onclick={handleTestConnection}
      disabled={!serverUrl.trim() || !username.trim() || !password.trim() || accountStore.testing || saving}
    >
      <span class="material-icons" style="font-size: 18px; margin-right: 8px;">
        {accountStore.testing ? 'hourglass_empty' : 'wifi_tethering'}
      </span>
      {accountStore.testing ? 'Testing...' : 'Test Connection'}
    </Button>

    {#if accountStore.testResult}
      {#if accountStore.testResult.success}
        <div class="test-result success">
          <span class="material-icons">check_circle</span>
          <span>Connection successful!</span>
        </div>
        {#if discoveredCalendars.length > 0}
          <div class="calendars">
            <span class="calendars-label">Discovered calendars with tasks:</span>
            {#if todoCalendars.length > 0}
              <List dense>
                {#each todoCalendars as cal}
                  <Item>
                    {#if cal.color}
                      <Graphic>
                        <span class="cal-color" style:background-color={cal.color}></span>
                      </Graphic>
                    {/if}
                    <Text>{cal.display_name || cal.href}</Text>
                  </Item>
                {/each}
              </List>
            {:else}
              <p class="no-calendars">No calendars with task support found</p>
            {/if}
          </div>
        {/if}
      {:else}
        <div class="test-result error">
          <span class="material-icons">error</span>
          <span>{accountStore.testResult.error || 'Connection failed'}</span>
        </div>
      {/if}
    {/if}
  </div>

  {#if accountStore.error && !accountStore.testResult}
    <p class="error">{accountStore.error}</p>
  {/if}

  <div class="form-actions">
    <Button variant="text" onclick={onClose} disabled={saving}>
      Cancel
    </Button>
    <Button
      variant="raised"
      type="submit"
      disabled={!name.trim() || !serverUrl.trim() || !username.trim() || !password.trim() || saving || (!isEditing && !connectionValid)}
      title={!isEditing && !connectionValid ? 'Test connection first' : ''}
    >
      {#if saving && isEditing}
        Saving...
      {:else if saving}
        Adding & Syncing...
      {:else if isEditing}
        Save
      {:else}
        Add Account
      {/if}
    </Button>
  </div>
  {#if !isEditing && !connectionValid && name.trim() && serverUrl.trim() && username.trim() && password.trim()}
    <p class="hint">Please test the connection before adding the account.</p>
  {/if}
</form>

<style>
  .account-form {
    display: flex;
    flex-direction: column;
    gap: var(--app-spacing-4);
    padding: var(--app-spacing-6);
  }

  h3 {
    margin: 0 0 var(--app-spacing-2);
    font-size: var(--app-font-size-lg);
    font-weight: 500;
  }

  .test-section {
    display: flex;
    flex-direction: column;
    gap: var(--app-spacing-3);
    padding-top: var(--app-spacing-2);
  }

  .test-result {
    display: flex;
    align-items: center;
    gap: var(--app-spacing-2);
    padding: var(--app-spacing-3) var(--app-spacing-4);
    border-radius: var(--app-radius-md);
    font-size: var(--app-font-size-sm);
  }

  .test-result.success {
    background: var(--app-color-status-success-bg);
    color: var(--app-color-status-success-text);
  }

  .test-result.error {
    background: var(--app-color-status-error-bg);
    color: var(--app-color-status-error-text);
  }

  .test-result .material-icons {
    font-size: 20px;
  }

  .calendars {
    padding: var(--app-spacing-3) var(--app-spacing-4);
    background: var(--mdc-theme-surface, #fff);
    border-radius: var(--app-radius-md);
    font-size: var(--app-font-size-sm);
    border: 1px solid var(--app-color-border);
  }

  .calendars-label {
    display: block;
    margin-bottom: var(--app-spacing-2);
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .cal-color {
    width: 12px;
    height: 12px;
    border-radius: 3px;
    display: inline-block;
  }

  .no-calendars {
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
    font-style: italic;
    margin: 0;
  }

  .error {
    color: var(--mdc-theme-error, #dc2626);
    font-size: var(--app-font-size-sm);
    margin: 0;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--app-spacing-2);
    padding-top: var(--app-spacing-2);
  }

  .hint {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
    text-align: right;
  }
</style>
