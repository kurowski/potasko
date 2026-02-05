// Preferences store using Svelte 5 runes
import * as api from '$lib/api';
import { listStore } from './lists.svelte';

// Reactive state
let showLocalAccounts = $state(true);
let loading = $state(false);
let error = $state<string | null>(null);

// Actions
async function load() {
  loading = true;
  error = null;
  try {
    showLocalAccounts = await api.getShowLocalAccounts();
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
  } finally {
    loading = false;
  }
}

async function setShowLocalAccounts(enabled: boolean) {
  error = null;
  try {
    await api.setShowLocalAccounts(enabled);
    showLocalAccounts = enabled;

    // If toggling OFF and a local list is selected, switch to Today view
    if (!enabled) {
      const selectedList = listStore.selectedList;
      if (selectedList && selectedList.account_id === null) {
        listStore.selectSpecial('today');
      }
    }
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    throw e;
  }
}

// Export reactive getters and actions
export const preferencesStore = {
  get showLocalAccounts() { return showLocalAccounts; },
  get loading() { return loading; },
  get error() { return error; },
  load,
  setShowLocalAccounts,
};
