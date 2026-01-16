<script lang="ts">
  import { onMount } from 'svelte';
  import type { Task } from '$lib/types';
  import { listStore } from '$lib/stores/lists.svelte';
  import { taskStore } from '$lib/stores/tasks.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import TaskListView from '$lib/components/TaskListView.svelte';
  import TaskForm from '$lib/components/TaskForm.svelte';
  import AccountList from '$lib/components/AccountList.svelte';

  let editingTask = $state<Task | null>(null);
  let showSettings = $state(false);

  // Mobile responsiveness state
  const MOBILE_BREAKPOINT = 768;
  let isMobile = $state(false);
  let sidebarOpen = $state(false);

  function checkMobile() {
    const wasMobile = isMobile;
    isMobile = window.innerWidth < MOBILE_BREAKPOINT;
    // Close sidebar when switching to desktop mode
    if (wasMobile && !isMobile) {
      sidebarOpen = false;
    }
  }

  function toggleSidebar() {
    sidebarOpen = !sidebarOpen;
  }

  function closeSidebar() {
    sidebarOpen = false;
  }

  // Get current view title for mobile header
  const currentViewTitle = $derived(() => {
    if (showSettings) return 'Settings';
    const view = listStore.selectedView;
    if (!view) return 'Potasko';
    if (view.type === 'special') {
      return view.view === 'today' ? 'Today' : 'Overdue';
    }
    const list = listStore.lists.find(l => l.id === view.id);
    return list?.name ?? 'Tasks';
  });

  // Load lists on mount
  onMount(() => {
    listStore.load();
    checkMobile();
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
  });

  // Load tasks when selected view changes
  $effect(() => {
    const view = listStore.selectedView;
    if (view?.type === 'list') {
      taskStore.loadList(view.id);
    } else if (view?.type === 'special') {
      taskStore.loadSpecial(view.view);
    } else {
      taskStore.clear();
    }
  });

  // Determine if we can add tasks (only in list view)
  const canAddTasks = $derived(listStore.selectedView?.type === 'list');

  function handleEditTask(task: Task) {
    editingTask = task;
  }

  function handleCloseEdit() {
    editingTask = null;
  }

  function handleToggleSettings() {
    showSettings = !showSettings;
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="app-layout" class:mobile={isMobile}>
  {#if isMobile}
    <header class="mobile-header">
      <button class="hamburger-btn" onclick={toggleSidebar} aria-label="Toggle menu">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="3" y1="12" x2="21" y2="12"></line>
          <line x1="3" y1="6" x2="21" y2="6"></line>
          <line x1="3" y1="18" x2="21" y2="18"></line>
        </svg>
      </button>
      <h1>{currentViewTitle()}</h1>
    </header>
  {/if}

  <Sidebar
    {showSettings}
    onToggleSettings={handleToggleSettings}
    {isMobile}
    isOpen={sidebarOpen}
    onClose={closeSidebar}
  />

  {#if isMobile && sidebarOpen}
    <div class="backdrop visible" onclick={closeSidebar}></div>
  {/if}

  <main class="main-content">
    {#if showSettings}
      <div class="settings-panel">
        <AccountList />
      </div>
    {:else if listStore.selectedView}
      <TaskListView onEditTask={handleEditTask} />
      {#if canAddTasks}
        {#key editingTask?.id}
          <TaskForm task={editingTask} onClose={handleCloseEdit} />
        {/key}
      {/if}
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
    height: 100%;
    overflow: hidden;
  }

  .app-layout.mobile {
    flex-direction: column;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
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

  .settings-panel {
    flex: 1;
    padding: 1.5rem;
    overflow-y: auto;
    background: var(--bg-primary);
  }

  @media (max-width: 768px) {
    .settings-panel {
      padding: 1rem;
    }
  }
</style>
