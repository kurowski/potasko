<script lang="ts">
  import type { Task } from '$lib/types';
  import { taskStore } from '$lib/stores/tasks.svelte';
  import { listStore } from '$lib/stores/lists.svelte';
  import TaskItem from './TaskItem.svelte';
  import SyncStatus from './SyncStatus.svelte';
  import Button from '@smui/button';
  import CircularProgress from '@smui/circular-progress';

  interface Props {
    onEditTask?: (task: Task) => void;
    onAddTask?: () => void;
  }

  let { onEditTask, onAddTask }: Props = $props();

  // Separate completed and incomplete tasks
  const incompleteTasks = $derived(taskStore.tasks.filter(t => !t.completed));
  const completedTasks = $derived(taskStore.tasks.filter(t => t.completed));

  // Get the title based on view type
  const viewTitle = $derived(() => {
    const view = listStore.selectedView;
    if (view?.type === 'list') {
      return listStore.selectedList?.name ?? 'Tasks';
    } else if (view?.type === 'special') {
      return view.view === 'today' ? 'Today' : 'Overdue';
    }
    return 'Tasks';
  });

  // Hide completed section only for Overdue view (Today shows completed tasks)
  const hideCompletedSection = $derived(listStore.selectedSpecialView === 'overdue');

  // Get list ID for sync (only for list views, not special views)
  const syncListId = $derived(
    listStore.selectedView?.type === 'list' ? listStore.selectedList?.id : null
  );

  // Show add button only for list views (not special views)
  const canAddTask = $derived(listStore.selectedView?.type === 'list');
</script>

<div class="task-list-view">
  <header class="list-header">
    <div class="header-top">
      <h1>{viewTitle()}</h1>
      <div class="header-actions">
        {#if canAddTask}
          <Button variant="raised" onclick={onAddTask}>
            <span class="material-icons" style="font-size: 18px; margin-right: 4px;">add</span>
            Add Task
          </Button>
        {/if}
        {#if syncListId}
          <SyncStatus listId={syncListId} />
        {/if}
      </div>
    </div>
    <span class="task-count">{incompleteTasks.length} tasks</span>
  </header>

  {#if taskStore.loading}
    <div class="loading">
      <CircularProgress indeterminate />
      <p>Loading tasks...</p>
    </div>
  {:else if taskStore.error}
    <div class="error">
      <span class="material-icons">error</span>
      {taskStore.error}
    </div>
  {:else}
    <div class="tasks-container">
      {#if incompleteTasks.length === 0 && completedTasks.length === 0}
        <div class="empty-state">
          <span class="material-icons empty-icon">task_alt</span>
          <p>No tasks yet</p>
          {#if canAddTask}
            <p class="hint">Click "Add Task" to get started</p>
          {/if}
        </div>
      {:else}
        <div class="task-section">
          {#each incompleteTasks as task (task.id)}
            <TaskItem {task} onEdit={onEditTask} />
          {/each}
        </div>

        {#if !hideCompletedSection && completedTasks.length > 0}
          <details class="completed-section">
            <summary>
              <span class="material-icons expand-icon">expand_more</span>
              Completed ({completedTasks.length})
            </summary>
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
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }

  .list-header {
    padding: 1.5rem 1.5rem 1rem;
    border-bottom: 1px solid rgba(0, 0, 0, 0.12);
  }

  @media (prefers-color-scheme: dark) {
    .list-header {
      border-bottom-color: rgba(255, 255, 255, 0.12);
    }
  }

  .header-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .list-header h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 500;
  }

  .task-count {
    font-size: 0.875rem;
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
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
    border-top: 1px solid rgba(0, 0, 0, 0.12);
    padding-top: 1rem;
  }

  @media (prefers-color-scheme: dark) {
    .completed-section {
      border-top-color: rgba(255, 255, 255, 0.12);
    }
  }

  .completed-section summary {
    cursor: pointer;
    font-size: 0.875rem;
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
    padding: 0.5rem 0;
    user-select: none;
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .completed-section summary:hover {
    color: var(--mdc-theme-on-surface, #000);
  }

  .expand-icon {
    font-size: 20px;
    transition: transform 0.2s;
  }

  .completed-section[open] .expand-icon {
    transform: rotate(180deg);
  }

  .completed-section summary::-webkit-details-marker {
    display: none;
  }

  .loading {
    padding: 3rem;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .loading p {
    margin: 0;
  }

  .error {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 2rem;
    color: var(--mdc-theme-error, #dc2626);
  }

  .empty-state {
    padding: 3rem 2rem;
    text-align: center;
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .empty-icon {
    font-size: 48px;
    opacity: 0.5;
    margin-bottom: 0.5rem;
  }

  .empty-state p {
    margin: 0.25rem 0;
  }

  .empty-state .hint {
    font-size: 0.875rem;
  }

  /* Mobile adjustments */
  @media (max-width: 768px) {
    .list-header {
      display: none; /* Title shown in mobile header */
    }

    .tasks-container {
      padding: 0.75rem 1rem;
      padding-bottom: env(safe-area-inset-bottom, 0.75rem);
    }
  }
</style>
