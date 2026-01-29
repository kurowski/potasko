<script lang="ts">
  import type { Task } from '$lib/types';
  import { taskStore } from '$lib/stores/tasks.svelte';
  import Checkbox from '@smui/checkbox';
  import IconButton from '@smui/icon-button';

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

  function getRecurrenceLabel(rrule: string | null): string {
    if (!rrule) return '';
    if (rrule.includes('FREQ=DAILY')) return 'Daily';
    if (rrule.includes('FREQ=WEEKLY')) return 'Weekly';
    if (rrule.includes('FREQ=MONTHLY')) return 'Monthly';
    if (rrule.includes('FREQ=YEARLY')) return 'Yearly';
    return 'Repeats';
  }

  const isOverdue = $derived(
    task.due_date && !task.completed && new Date(task.due_date) < new Date()
  );
</script>

<div class="task-item" class:completed={task.completed}>
  <div class="checkbox-wrapper">
    <Checkbox
      checked={task.completed}
      onclick={handleToggle}
      touch
    />
  </div>

  <button class="task-content" onclick={() => onEdit?.(task)}>
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
      {#if task.rrule}
        <span class="recurrence">
          <span class="material-icons recurrence-icon">repeat</span>
          {getRecurrenceLabel(task.rrule)}
        </span>
      {/if}
    </div>
  </button>

  <div class="delete-wrapper">
    <IconButton onclick={handleDelete} title="Delete task">
      <span class="material-icons">delete</span>
    </IconButton>
  </div>
</div>

<style>
  .task-item {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.5rem;
    border-radius: 8px;
    transition: background-color 0.15s;
  }

  .task-item:hover {
    background: var(--mdc-theme-surface, #fff);
  }

  .task-item.completed {
    opacity: 0.6;
  }

  .checkbox-wrapper {
    flex-shrink: 0;
  }

  .task-content {
    flex: 1;
    min-width: 0;
    cursor: pointer;
    background: none;
    border: none;
    padding: 0.25rem 0;
    font: inherit;
    text-align: left;
    color: inherit;
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
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.25rem;
    font-size: 0.75rem;
  }

  .due-date {
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .due-date.overdue {
    color: var(--mdc-theme-error, #dc2626);
    font-weight: 500;
  }

  .priority {
    padding: 0.125rem 0.5rem;
    border-radius: 12px;
    font-weight: 500;
    font-size: 0.6875rem;
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  .priority-high {
    background: #fecaca;
    color: #991b1b;
  }

  .priority-medium {
    background: #fef3c7;
    color: #92400e;
  }

  .priority-low {
    background: #dbeafe;
    color: #1e40af;
  }

  :global(.dark) .priority-high {
    background: #7f1d1d;
    color: #fecaca;
  }

  :global(.dark) .priority-medium {
    background: #78350f;
    color: #fef3c7;
  }

  :global(.dark) .priority-low {
    background: #1e3a8a;
    color: #dbeafe;
  }

  .recurrence {
    display: flex;
    align-items: center;
    gap: 0.125rem;
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .recurrence-icon {
    font-size: 14px;
  }

  .delete-wrapper {
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .task-item:hover .delete-wrapper {
    opacity: 1;
  }

  /* Mobile touch target improvements */
  @media (max-width: 768px) {
    .task-item {
      padding: 0.25rem;
    }

    .delete-wrapper {
      opacity: 1; /* Always visible on mobile */
    }
  }

  /* Dark mode via prefers-color-scheme */
  @media (prefers-color-scheme: dark) {
    .priority-high {
      background: #7f1d1d;
      color: #fecaca;
    }

    .priority-medium {
      background: #78350f;
      color: #fef3c7;
    }

    .priority-low {
      background: #1e3a8a;
      color: #dbeafe;
    }
  }
</style>
