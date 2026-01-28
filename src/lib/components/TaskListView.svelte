<script lang="ts">
  import type { Task } from '$lib/types';
  import { taskStore } from '$lib/stores/tasks.svelte';
  import { listStore } from '$lib/stores/lists.svelte';
  import TaskItem from './TaskItem.svelte';
  import SyncStatus from './SyncStatus.svelte';

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

  // Can add tasks only in list view (not special views like Today/Overdue)
  const canAddTasks = $derived(listStore.selectedView?.type === 'list');
</script>

<div class="flex-1 flex flex-col h-full min-h-0 overflow-hidden">
  <!-- Header (hidden on mobile, shown in mobile header instead) -->
  <header class="hidden md:block px-6 pt-6 pb-4 border-b border-surface-300-700">
    <div class="flex items-center justify-between gap-4">
      <div>
        <h1 class="m-0 text-2xl font-semibold">{viewTitle()}</h1>
        <span class="text-sm text-surface-500">{incompleteTasks.length} tasks</span>
      </div>
      <div class="flex items-center gap-3">
        {#if syncListId}
          <SyncStatus listId={syncListId} />
        {/if}
        {#if canAddTasks && onAddTask}
          <button
            class="btn preset-filled-primary-500"
            onclick={onAddTask}
          >
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
            Add Task
          </button>
        {/if}
      </div>
    </div>
  </header>

  {#if taskStore.loading}
    <div class="p-8 text-center text-surface-500">Loading tasks...</div>
  {:else if taskStore.error}
    <div class="p-8 text-center text-error-500">{taskStore.error}</div>
  {:else}
    <div class="flex-1 overflow-y-auto px-4 py-3 md:px-6 md:py-4 pb-[calc(0.75rem+var(--safe-area-bottom))]">
      {#if incompleteTasks.length === 0 && completedTasks.length === 0}
        <div class="p-8 text-center text-surface-500">
          <p class="my-1">No tasks yet</p>
          <p class="my-1 text-sm">Add a task to get started</p>
        </div>
      {:else}
        <div class="flex flex-col">
          {#each incompleteTasks as task (task.id)}
            <TaskItem {task} onEdit={onEditTask} />
          {/each}
        </div>

        {#if !hideCompletedSection && completedTasks.length > 0}
          <details class="mt-6 pt-4 border-t border-surface-300-700">
            <summary class="cursor-pointer text-sm text-surface-500 py-2 select-none hover:text-surface-700 dark:hover:text-surface-300">
              Completed ({completedTasks.length})
            </summary>
            <div class="flex flex-col">
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
