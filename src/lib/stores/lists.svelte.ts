// Task lists store using Svelte 5 runes
import type { TaskList, CreateTaskList, UpdateTaskList } from '$lib/types';
import * as api from '$lib/api';

// Special view types
export type SpecialView = 'today' | 'overdue';
export type ViewSelection = { type: 'list'; id: number } | { type: 'special'; view: SpecialView };

// Reactive state
let lists = $state<TaskList[]>([]);
let selectedView = $state<ViewSelection | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);

// Derived state - for backwards compatibility and convenience
const selectedListId = $derived(() => {
  const view = selectedView;
  return view?.type === 'list' ? view.id : null;
});
const selectedList = $derived(() => {
  const view = selectedView;
  if (view?.type === 'list') {
    return lists.find(l => l.id === view.id) ?? null;
  }
  return null;
});
const selectedSpecialView = $derived(() => {
  const view = selectedView;
  return view?.type === 'special' ? view.view : null;
});

// Actions
async function load() {
  loading = true;
  error = null;
  try {
    lists = await api.getLists();
    // Select first list if none selected
    if (selectedView === null && lists.length > 0) {
      selectedView = { type: 'list', id: lists[0].id };
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
    selectedView = { type: 'list', id: newList.id };
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
    if (selectedView?.type === 'list' && selectedView.id === id) {
      selectedView = lists.length > 0 ? { type: 'list', id: lists[0].id } : null;
    }
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    throw e;
  }
}

function select(id: number) {
  selectedView = { type: 'list', id };
}

function selectSpecial(view: SpecialView) {
  selectedView = { type: 'special', view };
}

// Export reactive getters and actions
export const listStore = {
  get lists() { return lists; },
  get selectedListId() { return selectedListId(); },
  get selectedList() { return selectedList(); },
  get selectedSpecialView() { return selectedSpecialView(); },
  get selectedView() { return selectedView; },
  get loading() { return loading; },
  get error() { return error; },
  load,
  create,
  update,
  remove,
  select,
  selectSpecial,
};
