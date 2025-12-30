// Tasks store using Svelte 5 runes
import type { Task, CreateTask, UpdateTask } from '$lib/types';
import type { SpecialView } from '$lib/stores/lists.svelte';
import * as api from '$lib/api';

// Reactive state
let tasks = $state<Task[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);

// Actions
async function loadList(listId: number) {
  loading = true;
  error = null;
  try {
    tasks = await api.getTasks(listId);
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    tasks = [];
  } finally {
    loading = false;
  }
}

async function loadSpecial(view: SpecialView) {
  loading = true;
  error = null;
  try {
    if (view === 'today') {
      tasks = await api.getTasksToday();
    } else if (view === 'overdue') {
      tasks = await api.getTasksOverdue();
    }
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    tasks = [];
  } finally {
    loading = false;
  }
}

async function create(data: CreateTask) {
  error = null;
  try {
    const newTask = await api.createTask(data);
    tasks = [...tasks, newTask];
    return newTask;
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    throw e;
  }
}

async function update(id: number, data: UpdateTask) {
  error = null;
  try {
    const updated = await api.updateTask(id, data);
    tasks = tasks.map(t => t.id === id ? updated : t);
    return updated;
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    throw e;
  }
}

async function toggle(id: number) {
  error = null;
  try {
    const updated = await api.toggleTaskCompletion(id);
    tasks = tasks.map(t => t.id === id ? updated : t);
    return updated;
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    throw e;
  }
}

async function remove(id: number) {
  error = null;
  try {
    await api.deleteTask(id);
    tasks = tasks.filter(t => t.id !== id);
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    throw e;
  }
}

function clear() {
  tasks = [];
}

// Export reactive getters and actions
export const taskStore = {
  get tasks() { return tasks; },
  get loading() { return loading; },
  get error() { return error; },
  loadList,
  loadSpecial,
  create,
  update,
  toggle,
  remove,
  clear,
};
