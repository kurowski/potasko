// Tauri command wrappers
import { invoke } from '@tauri-apps/api/core';
import type {
  Task,
  TaskList,
  CreateTask,
  UpdateTask,
  CreateTaskList,
  UpdateTaskList,
} from '$lib/types';

// --- List commands ---

export async function getLists(): Promise<TaskList[]> {
  return invoke('get_lists');
}

export async function getList(id: number): Promise<TaskList> {
  return invoke('get_list', { id });
}

export async function createList(data: CreateTaskList): Promise<TaskList> {
  return invoke('create_list', { data });
}

export async function updateList(id: number, data: UpdateTaskList): Promise<TaskList> {
  return invoke('update_list', { id, data });
}

export async function deleteList(id: number): Promise<void> {
  return invoke('delete_list', { id });
}

// --- Task commands ---

export async function getTasks(listId: number): Promise<Task[]> {
  return invoke('get_tasks', { listId });
}

export async function getTask(id: number): Promise<Task> {
  return invoke('get_task', { id });
}

export async function createTask(data: CreateTask): Promise<Task> {
  return invoke('create_task', { data });
}

export async function updateTask(id: number, data: UpdateTask): Promise<Task> {
  return invoke('update_task', { id, data });
}

export async function deleteTask(id: number): Promise<void> {
  return invoke('delete_task', { id });
}

export async function toggleTaskCompletion(id: number): Promise<Task> {
  return invoke('toggle_task_completion', { id });
}

export async function getTasksToday(): Promise<Task[]> {
  return invoke('get_tasks_today');
}

export async function getTasksOverdue(): Promise<Task[]> {
  return invoke('get_tasks_overdue');
}

// --- Account commands ---

import type {
  Account,
  CreateAccount,
  UpdateAccount,
  AccountTestResult,
} from '$lib/types';

export async function getAccounts(): Promise<Account[]> {
  return invoke('get_accounts');
}

export async function getAccount(id: number): Promise<Account> {
  return invoke('get_account', { id });
}

export async function createAccount(data: CreateAccount): Promise<Account> {
  return invoke('create_account', { data });
}

export async function updateAccount(id: number, data: UpdateAccount): Promise<Account> {
  return invoke('update_account', { id, data });
}

export async function deleteAccount(id: number): Promise<void> {
  return invoke('delete_account', { id });
}

export async function testAccountConnection(
  serverUrl: string,
  username: string,
  password: string
): Promise<AccountTestResult> {
  return invoke('test_account_connection', { serverUrl, username, password });
}
