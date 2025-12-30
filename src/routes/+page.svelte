<script lang="ts">
  import { onMount } from 'svelte';
  import type { Task } from '$lib/types';
  import { listStore } from '$lib/stores/lists.svelte';
  import { taskStore } from '$lib/stores/tasks.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import TaskListView from '$lib/components/TaskListView.svelte';
  import TaskForm from '$lib/components/TaskForm.svelte';

  let editingTask = $state<Task | null>(null);

  // Load lists on mount
  onMount(() => {
    listStore.load();
  });

  // Load tasks when selected list changes
  $effect(() => {
    const listId = listStore.selectedListId;
    if (listId) {
      taskStore.load(listId);
    } else {
      taskStore.clear();
    }
  });

  function handleEditTask(task: Task) {
    editingTask = task;
  }

  function handleCloseEdit() {
    editingTask = null;
  }
</script>

<div class="app-layout">
  <Sidebar />

  <main class="main-content">
    {#if listStore.selectedListId}
      <TaskListView onEditTask={handleEditTask} />
      <TaskForm task={editingTask} onClose={handleCloseEdit} />
    {:else if listStore.loading}
      <div class="loading-state">Loading...</div>
    {:else}
      <div class="empty-state">
        <p>No lists yet</p>
        <p>Create a list to get started</p>
      </div>
    {/if}
  </main>
</div>

<style>
  .app-layout {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .loading-state,
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
  }

  .empty-state p {
    margin: 0.25rem 0;
  }
</style>
