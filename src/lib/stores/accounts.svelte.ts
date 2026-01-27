// CalDAV accounts store using Svelte 5 runes
import type { Account, CreateAccount, UpdateAccount, AccountTestResult, CreateAccountResult } from '$lib/types';
import * as api from '$lib/api';

// Reactive state
let accounts = $state<Account[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);
let testing = $state(false);
let testResult = $state<AccountTestResult | null>(null);

// Actions
async function load() {
  loading = true;
  error = null;
  try {
    accounts = await api.getAccounts();
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
  } finally {
    loading = false;
  }
}

async function create(data: CreateAccount) {
  error = null;
  try {
    const newAccount = await api.createAccount(data);
    accounts = [...accounts, newAccount];
    return newAccount;
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    throw e;
  }
}

async function createAndSync(data: CreateAccount): Promise<CreateAccountResult> {
  error = null;
  try {
    const result = await api.createAccountAndSync(data);
    accounts = [...accounts, result.account];
    return result;
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    throw e;
  }
}

async function update(id: number, data: UpdateAccount) {
  error = null;
  try {
    const updated = await api.updateAccount(id, data);
    accounts = accounts.map(a => a.id === id ? updated : a);
    return updated;
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    throw e;
  }
}

async function remove(id: number) {
  error = null;
  try {
    await api.deleteAccount(id);
    accounts = accounts.filter(a => a.id !== id);
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    throw e;
  }
}

async function testConnection(serverUrl: string, username: string, password: string) {
  testing = true;
  testResult = null;
  error = null;
  try {
    testResult = await api.testAccountConnection(serverUrl, username, password);
    return testResult;
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    testResult = {
      success: false,
      principal_url: null,
      calendar_home_url: null,
      calendars: [],
      error: error,
    };
    throw e;
  } finally {
    testing = false;
  }
}

function clearTestResult() {
  testResult = null;
}

// Export reactive getters and actions
export const accountStore = {
  get accounts() { return accounts; },
  get loading() { return loading; },
  get error() { return error; },
  get testing() { return testing; },
  get testResult() { return testResult; },
  load,
  create,
  createAndSync,
  update,
  remove,
  testConnection,
  clearTestResult,
};
