<script lang="ts">
  import type { Task } from '$lib/types';
  import { taskStore } from '$lib/stores/tasks.svelte';
  import { listStore } from '$lib/stores/lists.svelte';
  import TaskItem from './TaskItem.svelte';
  import SyncStatus from './SyncStatus.svelte';
  import { Accordion, AccordionItem, InlineLoading } from 'carbon-components-svelte';

  interface Props {
    onEditTask?: (task: Task) => void;
  }

  let { onEditTask }: Props = $props();

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
</script>

<div class="task-list-view">
  <header class="list-header">
    <div class="header-top">
      <h1>{viewTitle()}</h1>
      {#if syncListId}
        <SyncStatus listId={syncListId} />
      {/if}
    </div>
    <span class="task-count">{incompleteTasks.length} task{incompleteTasks.length !== 1 ? 's' : ''}</span>
  </header>

  {#if taskStore.loading}
    <div class="loading">
      <InlineLoading description="Loading tasks..." />
    </div>
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

        {#if !hideCompletedSection && completedTasks.length > 0}
          <Accordion>
            <AccordionItem title="Completed ({completedTasks.length})">
              <div class="task-section">
                {#each completedTasks as task (task.id)}
                  <TaskItem {task} onEdit={onEditTask} />
                {/each}
              </div>
            </AccordionItem>
          </Accordion>
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
    padding: 1rem;
    border-bottom: 1px solid var(--cds-border-subtle, var(--border-color));
  }

  .header-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .list-header h1 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .task-count {
    font-size: 0.875rem;
    color: var(--cds-text-secondary, var(--text-secondary));
  }

  .tasks-container {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem 1rem;
  }

  .task-section {
    display: flex;
    flex-direction: column;
  }

  .loading,
  .error,
  .empty-state {
    padding: 2rem;
    text-align: center;
  }

  .error {
    color: var(--cds-danger, var(--error-color));
  }

  .empty-state {
    color: var(--cds-text-secondary, var(--text-secondary));
  }

  .empty-state p {
    margin: 0.25rem 0;
  }

  .empty-state .hint {
    font-size: 0.875rem;
  }

  /* Hide header on mobile - shown in Carbon Header instead */
  @media (max-width: 1056px) {
    .list-header {
      display: none;
    }

    .tasks-container {
      padding: 0.5rem;
      padding-bottom: calc(0.5rem + env(safe-area-inset-bottom, 0));
    }
  }
</style>
