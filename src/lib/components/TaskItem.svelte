<script lang="ts">
  import type { Task } from '$lib/types';
  import { taskStore } from '$lib/stores/tasks.svelte';
  import { Checkbox, Tag, Button } from 'carbon-components-svelte';
  import { TrashCan, Renew } from 'carbon-icons-svelte';

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

  function getPriorityTagType(priority: number | null): 'red' | 'warm-gray' | 'blue' | 'gray' {
    if (priority === null) return 'gray';
    if (priority <= 3) return 'red';
    if (priority <= 6) return 'warm-gray';
    return 'blue';
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
  <Checkbox
    checked={task.completed}
    on:check={handleToggle}
    labelText=""
    hideLabel
  />

  <button class="task-content" onclick={() => onEdit?.(task)}>
    <span class="task-title">{task.title}</span>
    <div class="task-meta">
      {#if task.due_date}
        <Tag size="sm" type={isOverdue ? 'red' : 'gray'}>{formatDueDate(task.due_date)}</Tag>
      {/if}
      {#if task.priority}
        <Tag size="sm" type={getPriorityTagType(task.priority)}>{getPriorityLabel(task.priority)}</Tag>
      {/if}
      {#if task.rrule}
        <Tag size="sm" type="teal" icon={Renew}>{getRecurrenceLabel(task.rrule)}</Tag>
      {/if}
    </div>
  </button>

  <Button
    kind="ghost"
    size="small"
    iconDescription="Delete task"
    icon={TrashCan}
    on:click={handleDelete}
    class="delete-btn"
  />
</div>

<style>
  .task-item {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.5rem;
    border-radius: 0;
    transition: background-color 0.15s;
  }

  .task-item:hover {
    background: var(--cds-layer-hover, var(--bg-hover));
  }

  .task-item.completed {
    opacity: 0.6;
  }

  .task-content {
    flex: 1;
    min-width: 0;
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    text-align: left;
    color: inherit;
  }

  .task-title {
    display: block;
    font-size: 0.875rem;
    line-height: 1.4;
  }

  .completed .task-title {
    text-decoration: line-through;
  }

  .task-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    margin-top: 0.25rem;
  }

  /* Hide delete button until hover on desktop */
  .task-item :global(.delete-btn) {
    opacity: 0;
    transition: opacity 0.15s;
  }

  .task-item:hover :global(.delete-btn) {
    opacity: 1;
  }

  /* Mobile: always show delete, larger touch targets */
  @media (max-width: 1056px) {
    .task-item :global(.delete-btn) {
      opacity: 1;
    }
  }
</style>
