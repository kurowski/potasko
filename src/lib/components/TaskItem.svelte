<script lang="ts">
  import type { Task } from '$lib/types';
  import { taskStore } from '$lib/stores/tasks.svelte';

  interface Props {
    task: Task;
    onEdit?: (task: Task) => void;
  }

  let { task, onEdit }: Props = $props();

  async function handleToggle() {
    await taskStore.toggle(task.id);
  }

  async function handleDelete() {
    if (confirm(`Delete "${task.title}"?`)) {
      await taskStore.remove(task.id);
    }
  }

  function formatDueDate(dateStr: string | null): string {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const tomorrow = new Date(today);
    tomorrow.setDate(tomorrow.getDate() + 1);
    const taskDate = new Date(date);
    taskDate.setHours(0, 0, 0, 0);

    if (taskDate.getTime() === today.getTime()) return 'Today';
    if (taskDate.getTime() === tomorrow.getTime()) return 'Tomorrow';
    if (taskDate < today) return 'Overdue';

    return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
  }

  function getPriorityLabel(priority: number | null): string {
    if (priority === null) return '';
    if (priority <= 3) return 'High';
    if (priority <= 6) return 'Medium';
    return 'Low';
  }

  const isOverdue = $derived(
    task.due_date && !task.completed && new Date(task.due_date) < new Date()
  );
</script>

<div class="task-item" class:completed={task.completed}>
  <button
    class="checkbox"
    class:checked={task.completed}
    onclick={handleToggle}
    aria-label={task.completed ? 'Mark incomplete' : 'Mark complete'}
  >
    {#if task.completed}
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
      </svg>
    {/if}
  </button>

  <div class="task-content" onclick={() => onEdit?.(task)}>
    <span class="task-title">{task.title}</span>
    <div class="task-meta">
      {#if task.due_date}
        <span class="due-date" class:overdue={isOverdue}>
          {formatDueDate(task.due_date)}
        </span>
      {/if}
      {#if task.priority}
        <span class="priority priority-{getPriorityLabel(task.priority).toLowerCase()}">
          {getPriorityLabel(task.priority)}
        </span>
      {/if}
    </div>
  </div>

  <button class="delete-btn" onclick={handleDelete} title="Delete task">
    <svg viewBox="0 0 24 24" fill="currentColor">
      <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
    </svg>
  </button>
</div>

<style>
  .task-item {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.75rem;
    border-radius: 6px;
    transition: background-color 0.15s;
  }

  .task-item:hover {
    background: var(--bg-hover);
  }

  .task-item.completed {
    opacity: 0.6;
  }

  .checkbox {
    width: 20px;
    height: 20px;
    border: 2px solid var(--border-color);
    border-radius: 4px;
    background: transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-top: 2px;
  }

  .checkbox:hover {
    border-color: var(--accent-color);
  }

  .checkbox.checked {
    background: var(--accent-color);
    border-color: var(--accent-color);
    color: white;
  }

  .checkbox svg {
    width: 14px;
    height: 14px;
  }

  .task-content {
    flex: 1;
    min-width: 0;
    cursor: pointer;
  }

  .task-title {
    display: block;
    font-size: 0.9375rem;
  }

  .completed .task-title {
    text-decoration: line-through;
  }

  .task-meta {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.25rem;
    font-size: 0.75rem;
  }

  .due-date {
    color: var(--text-secondary);
  }

  .due-date.overdue {
    color: var(--error-color);
    font-weight: 500;
  }

  .priority {
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    font-weight: 500;
  }

  .priority-high {
    background: #fee2e2;
    color: #dc2626;
  }

  .priority-medium {
    background: #fef3c7;
    color: #d97706;
  }

  .priority-low {
    background: #dbeafe;
    color: #2563eb;
  }

  .delete-btn {
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    color: var(--text-secondary);
  }

  .task-item:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    background: var(--bg-hover);
    color: var(--error-color);
  }

  .delete-btn svg {
    width: 18px;
    height: 18px;
  }
</style>
