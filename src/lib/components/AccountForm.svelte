<script lang="ts">
  import { untrack } from 'svelte';
  import { accountStore } from '$lib/stores/accounts.svelte';
  import { listStore } from '$lib/stores/lists.svelte';
  import type { Account, CalendarInfo } from '$lib/types';

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

</script>

<svelte:window onkeydown={handleKeydown} />

<form class="card flex flex-col gap-4 p-6 bg-surface-100-900 rounded-lg" onsubmit={handleSubmit}>
  <h3 class="m-0 mb-2 text-lg font-semibold">{isEditing ? 'Edit Account' : 'Add CalDAV Account'}</h3>

  <label class="flex flex-col gap-1.5">
    <span class="text-sm text-surface-500">Account Name</span>
    <input
      type="text"
      class="input"
      bind:value={name}
      placeholder="e.g., Work, Personal"
      disabled={saving}
    />
  </label>

  <label class="flex flex-col gap-1.5">
    <span class="text-sm text-surface-500">Server URL</span>
    <input
      type="url"
      class="input"
      bind:value={serverUrl}
      placeholder="https://caldav.example.com"
      disabled={saving}
    />
  </label>

  <label class="flex flex-col gap-1.5">
    <span class="text-sm text-surface-500">Username</span>
    <input
      type="text"
      class="input"
      bind:value={username}
      placeholder="Username"
      disabled={saving}
    />
  </label>

  <label class="flex flex-col gap-1.5">
    <span class="text-sm text-surface-500">Password</span>
    <input
      type="password"
      class="input"
      bind:value={password}
      placeholder="Password"
      disabled={saving}
    />
  </label>

  <div class="flex flex-col gap-3 pt-2">
    <button
      type="button"
      class="btn preset-outlined self-start"
      onclick={handleTestConnection}
      disabled={!serverUrl.trim() || !username.trim() || !password.trim() || accountStore.testing || saving}
    >
      {accountStore.testing ? 'Testing...' : 'Test Connection'}
    </button>

    {#if accountStore.testResult}
      {#if accountStore.testResult.success}
        <div class="flex items-center gap-2 p-3 bg-success-500/20 rounded-md text-sm text-success-700 dark:text-success-300">
          <svg class="w-[18px] h-[18px] shrink-0" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
          </svg>
          <span>Connection successful!</span>
        </div>
        {#if discoveredCalendars.length > 0}
          <div class="p-3 bg-surface-50-950 rounded-md text-sm">
            <span class="block mb-2 text-surface-500">Discovered calendars with tasks:</span>
            <ul class="m-0 p-0 list-none flex flex-col gap-1.5">
              {#each discoveredCalendars.filter(c => c.supports_vtodo) as cal}
                <li class="flex items-center gap-2">
                  {#if cal.color}
                    <span class="w-3 h-3 rounded shrink-0" style:background-color={cal.color}></span>
                  {/if}
                  {cal.display_name || cal.href}
                </li>
              {/each}
              {#if discoveredCalendars.filter(c => c.supports_vtodo).length === 0}
                <li class="text-surface-500 italic">No calendars with task support found</li>
              {/if}
            </ul>
          </div>
        {/if}
      {:else}
        <div class="flex items-center gap-2 p-3 bg-error-500/20 rounded-md text-sm text-error-500">
          <svg class="w-[18px] h-[18px] shrink-0" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
          </svg>
          <span>{accountStore.testResult.error || 'Connection failed'}</span>
        </div>
      {/if}
    {/if}
  </div>

  {#if accountStore.error && !accountStore.testResult}
    <p class="m-0 text-sm text-error-500">{accountStore.error}</p>
  {/if}

  <div class="flex justify-end gap-3 pt-2">
    <button type="button" class="btn preset-outlined" onclick={onClose} disabled={saving}>Cancel</button>
    <button
      type="submit"
      class="btn preset-filled-primary-500"
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
    </button>
  </div>
  {#if !isEditing && !connectionValid && name.trim() && serverUrl.trim() && username.trim() && password.trim()}
    <p class="m-0 text-[0.8125rem] text-surface-500 text-right">Please test the connection before adding the account.</p>
  {/if}
</form>
