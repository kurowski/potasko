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

  function getPriorityClass(priority: number | null): string {
    if (priority === null) return '';
    if (priority <= 3) return 'preset-filled-error-500';
    if (priority <= 6) return 'preset-filled-warning-500';
    return 'preset-filled-primary-500';
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

<div
  class="group flex items-start gap-3 p-3 rounded-md transition-colors hover:bg-surface-100-900"
  class:opacity-60={task.completed}
>
  <!-- Checkbox -->
  <button
    class="shrink-0 mt-0.5 w-5 h-5 md:w-11 md:h-11 rounded border-2 flex items-center justify-center transition-colors
           {task.completed
             ? 'bg-primary-500 border-primary-500 text-white'
             : 'border-surface-400 hover:border-primary-500'}"
    onclick={handleToggle}
    aria-label={task.completed ? 'Mark incomplete' : 'Mark complete'}
  >
    {#if task.completed}
      <svg class="w-3.5 h-3.5 md:w-6 md:h-6" viewBox="0 0 24 24" fill="currentColor">
        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
      </svg>
    {/if}
  </button>

  <!-- Task Content -->
  <button
    class="flex-1 min-w-0 text-left bg-transparent border-none cursor-pointer p-0"
    onclick={() => onEdit?.(task)}
  >
    <span class="block text-[0.9375rem]" class:line-through={task.completed}>
      {task.title}
    </span>
    <div class="flex flex-wrap gap-2 mt-1 text-xs">
      {#if task.due_date}
        <span class:text-error-500={isOverdue} class:font-medium={isOverdue} class="text-surface-500">
          {formatDueDate(task.due_date)}
        </span>
      {/if}
      {#if task.priority}
        <span class="badge {getPriorityClass(task.priority)} text-xs px-1.5 py-0.5">
          {getPriorityLabel(task.priority)}
        </span>
      {/if}
      {#if task.rrule}
        <span class="flex items-center gap-1 text-surface-500">
          <svg class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
          </svg>
          {getRecurrenceLabel(task.rrule)}
        </span>
      {/if}
    </div>
  </button>

  <!-- Delete Button -->
  <button
    class="shrink-0 w-7 h-7 md:w-11 md:h-11 flex items-center justify-center rounded bg-transparent border-none cursor-pointer
           text-surface-500 opacity-0 md:opacity-100 group-hover:opacity-100 transition-opacity
           hover:bg-surface-200-800 hover:text-error-500"
    onclick={handleDelete}
    title="Delete task"
  >
    <svg class="w-[18px] h-[18px] md:w-6 md:h-6" viewBox="0 0 24 24" fill="currentColor">
      <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
    </svg>
  </button>
</div>
