<script lang="ts">
  import { untrack } from 'svelte';
  import { accountStore } from '$lib/stores/accounts.svelte';
  import { listStore } from '$lib/stores/lists.svelte';
  import type { Account, CalendarInfo } from '$lib/types';
  import {
    Form,
    TextInput,
    PasswordInput,
    Button,
    ButtonSet,
    InlineNotification,
    UnorderedList,
    ListItem,
    InlineLoading,
  } from 'carbon-components-svelte';
  import { Checkmark, WarningFilled } from 'carbon-icons-svelte';

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

  const taskCalendars = $derived(discoveredCalendars.filter(c => c.supports_vtodo));
</script>

<svelte:window onkeydown={handleKeydown} />

<Form on:submit={handleSubmit} class="account-form">
  <h3>{isEditing ? 'Edit Account' : 'Add CalDAV Account'}</h3>

  <TextInput
    bind:value={name}
    labelText="Account Name"
    placeholder="e.g., Work, Personal"
    disabled={saving}
    required
  />

  <TextInput
    bind:value={serverUrl}
    labelText="Server URL"
    placeholder="https://caldav.example.com"
    disabled={saving}
    type="url"
    required
  />

  <TextInput
    bind:value={username}
    labelText="Username"
    placeholder="Username"
    disabled={saving}
    required
  />

  <PasswordInput
    bind:value={password}
    labelText="Password"
    placeholder="Password"
    disabled={saving}
    required
  />

  <div class="test-section">
    <Button
      kind="tertiary"
      size="small"
      on:click={handleTestConnection}
      disabled={!serverUrl.trim() || !username.trim() || !password.trim() || accountStore.testing || saving}
    >
      {#if accountStore.testing}
        <InlineLoading description="Testing..." />
      {:else}
        Test Connection
      {/if}
    </Button>

    {#if accountStore.testResult}
      {#if accountStore.testResult.success}
        <InlineNotification
          kind="success"
          title="Success"
          subtitle="Connection successful!"
          hideCloseButton
        />
        {#if taskCalendars.length > 0}
          <div class="calendars">
            <span class="calendars-label">Discovered calendars with tasks:</span>
            <UnorderedList>
              {#each taskCalendars as cal}
                <ListItem>
                  {#if cal.color}
                    <span class="cal-color" style:background-color={cal.color}></span>
                  {/if}
                  {cal.display_name || cal.href}
                </ListItem>
              {/each}
            </UnorderedList>
          </div>
        {:else}
          <InlineNotification
            kind="warning"
            title="No task calendars"
            subtitle="No calendars with task support found"
            hideCloseButton
          />
        {/if}
      {:else}
        <InlineNotification
          kind="error"
          title="Failed"
          subtitle={accountStore.testResult.error || 'Connection failed'}
          hideCloseButton
        />
      {/if}
    {/if}
  </div>

  {#if accountStore.error && !accountStore.testResult}
    <InlineNotification
      kind="error"
      title="Error"
      subtitle={accountStore.error}
      hideCloseButton
    />
  {/if}

  <ButtonSet class="form-actions">
    <Button kind="secondary" on:click={onClose} disabled={saving}>
      Cancel
    </Button>
    <Button
      type="submit"
      disabled={!name.trim() || !serverUrl.trim() || !username.trim() || !password.trim() || saving || (!isEditing && !connectionValid)}
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
  </ButtonSet>

  {#if !isEditing && !connectionValid && name.trim() && serverUrl.trim() && username.trim() && password.trim()}
    <p class="hint">Please test the connection before adding the account.</p>
  {/if}
</Form>

<style>
  :global(.account-form) {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem;
  }

  h3 {
    margin: 0 0 0.5rem;
    font-size: 1.125rem;
    font-weight: 600;
  }

  .test-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding-top: 0.5rem;
  }

  .calendars {
    padding: 0.75rem;
    background: var(--cds-layer, var(--bg-primary));
    border-radius: 0;
    font-size: 0.875rem;
  }

  .calendars-label {
    display: block;
    margin-bottom: 0.5rem;
    color: var(--cds-text-secondary, var(--text-secondary));
  }

  .cal-color {
    width: 12px;
    height: 12px;
    border-radius: 3px;
    display: inline-block;
    margin-right: 0.5rem;
    vertical-align: middle;
  }

  :global(.form-actions) {
    justify-content: flex-end;
    padding-top: 0.5rem;
  }

  .hint {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--cds-text-secondary, var(--text-secondary));
    text-align: right;
  }
</style>
