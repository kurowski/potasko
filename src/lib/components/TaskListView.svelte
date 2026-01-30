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
    padding: var(--app-spacing-6) var(--app-spacing-6) var(--app-spacing-4);
    border-bottom: 1px solid var(--app-color-border);
  }

  .header-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--app-spacing-4);
    flex-wrap: wrap;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--app-spacing-3);
  }

  .list-header h1 {
    margin: 0;
    font-size: var(--app-font-size-xl);
    font-weight: 500;
  }

  .task-count {
    font-size: var(--app-font-size-sm);
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .tasks-container {
    flex: 1;
    overflow-y: auto;
    padding: var(--app-spacing-4) var(--app-spacing-6);
  }

  .task-section {
    display: flex;
    flex-direction: column;
  }

  .completed-section {
    margin-top: var(--app-spacing-6);
    border-top: 1px solid var(--app-color-border);
    padding-top: var(--app-spacing-4);
  }

  .completed-section summary {
    cursor: pointer;
    font-size: var(--app-font-size-sm);
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
    padding: var(--app-spacing-2) 0;
    user-select: none;
    display: flex;
    align-items: center;
    gap: var(--app-spacing-1);
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
    gap: var(--app-spacing-4);
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
  }

  .loading p {
    margin: 0;
  }

  .error {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--app-spacing-2);
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
    margin-bottom: var(--app-spacing-2);
  }

  .empty-state p {
    margin: var(--app-spacing-1) 0;
  }

  .empty-state .hint {
    font-size: var(--app-font-size-sm);
  }

  /* Mobile adjustments */
  @media (max-width: 768px) {
    .list-header {
      display: none; /* Title shown in mobile header */
    }

    .tasks-container {
      padding: var(--app-spacing-3) var(--app-spacing-4);
      padding-bottom: env(safe-area-inset-bottom, var(--app-spacing-3));
    }
  }
</style>
