<script lang="ts">
  import type { Task } from '$lib/types';
  import { taskStore } from '$lib/stores/tasks.svelte';
  import { listStore } from '$lib/stores/lists.svelte';
  import TaskItem from './TaskItem.svelte';

  interface Props {
    onEditTask?: (task: Task) => void;
  }

  let { onEditTask }: Props = $props();

  // Separate completed and incomplete tasks
  const incompleteTasks = $derived(taskStore.tasks.filter(t => !t.completed));
  const completedTasks = $derived(taskStore.tasks.filter(t => t.completed));
</script>

<div class="task-list-view">
  <header class="list-header">
    <h1>{listStore.selectedList?.name ?? 'Tasks'}</h1>
    <span class="task-count">{incompleteTasks.length} tasks</span>
  </header>

  {#if taskStore.loading}
    <div class="loading">Loading tasks...</div>
  {:else if taskStore.error}
    <div class="error">{taskStore.error}</div>
  {:else}
    <div class="tasks-container">
      {#if incompleteTasks.length === 0 && completedTasks.length === 0}
        <div class="empty-state">
          <p>No tasks yet</p>
          <p class="hint">Add a task to get started</p>
        </div>
      {:else}
        <div class="task-section">
          {#each incompleteTasks as task (task.id)}
            <TaskItem {task} onEdit={onEditTask} />
          {/each}
        </div>

        {#if completedTasks.length > 0}
          <details class="completed-section">
            <summary>Completed ({completedTasks.length})</summary>
            <div class="task-section">
              {#each completedTasks as task (task.id)}
                <TaskItem {task} onEdit={onEditTask} />
              {/each}
            </div>
          </details>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<style>
  .task-list-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .list-header {
    padding: 1.5rem 1.5rem 1rem;
    border-bottom: 1px solid var(--border-color);
  }

  .list-header h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .task-count {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .tasks-container {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.5rem;
  }

  .task-section {
    display: flex;
    flex-direction: column;
  }

  .completed-section {
    margin-top: 1.5rem;
    border-top: 1px solid var(--border-color);
    padding-top: 1rem;
  }

  .completed-section summary {
    cursor: pointer;
    font-size: 0.875rem;
    color: var(--text-secondary);
    padding: 0.5rem 0;
    user-select: none;
  }

  .completed-section summary:hover {
    color: var(--text-primary);
  }

  .loading,
  .error,
  .empty-state {
    padding: 2rem;
    text-align: center;
  }

  .error {
    color: var(--error-color);
  }

  .empty-state {
    color: var(--text-secondary);
  }

  .empty-state p {
    margin: 0.25rem 0;
  }

  .empty-state .hint {
    font-size: 0.875rem;
  }
</style>
