// Task lists store using Svelte 5 runes
import type { TaskList, CreateTaskList, UpdateTaskList } from '$lib/types';
import * as api from '$lib/api';

// Reactive state
let lists = $state<TaskList[]>([]);
let selectedListId = $state<number | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);

// Derived state
const selectedList = $derived(lists.find(l => l.id === selectedListId) ?? null);

// Actions
async function load() {
  loading = true;
  error = null;
  try {
    lists = await api.getLists();
    // Select first list if none selected
    if (selectedListId === null && lists.length > 0) {
      selectedListId = lists[0].id;
    }
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
  } finally {
    loading = false;
  }
}

async function create(data: CreateTaskList) {
  error = null;
  try {
    const newList = await api.createList(data);
    lists = [...lists, newList];
    selectedListId = newList.id;
    return newList;
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    throw e;
  }
}

async function update(id: number, data: UpdateTaskList) {
  error = null;
  try {
    const updated = await api.updateList(id, data);
    lists = lists.map(l => l.id === id ? updated : l);
    return updated;
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    throw e;
  }
}

async function remove(id: number) {
  error = null;
  try {
    await api.deleteList(id);
    lists = lists.filter(l => l.id !== id);
    // Select another list if we deleted the selected one
    if (selectedListId === id) {
      selectedListId = lists.length > 0 ? lists[0].id : null;
    }
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    throw e;
  }
}

function select(id: number) {
  selectedListId = id;
}

// Export reactive getters and actions
export const listStore = {
  get lists() { return lists; },
  get selectedListId() { return selectedListId; },
  get selectedList() { return selectedList; },
  get loading() { return loading; },
  get error() { return error; },
  load,
  create,
  update,
  remove,
  select,
};
