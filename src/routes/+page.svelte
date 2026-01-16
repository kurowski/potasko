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
  let showMobileForm = $state(false);

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
    if (isMobile) {
      showMobileForm = true;
    }
  }

  function handleCloseEdit() {
    editingTask = null;
    showMobileForm = false;
  }

  function handleToggleSettings() {
    showSettings = !showSettings;
  }

  function handleFabClick() {
    editingTask = null;
    showMobileForm = true;
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
        {#if !isMobile}
          <!-- Desktop: inline form -->
          {#key editingTask?.id}
            <TaskForm task={editingTask} onClose={handleCloseEdit} />
          {/key}
        {/if}
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

  <!-- Mobile: FAB and modal form -->
  {#if isMobile && canAddTasks && !showSettings}
    {#if showMobileForm}
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div class="form-modal-backdrop" onclick={handleCloseEdit}>
        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
        <div class="form-modal" onclick={(e) => e.stopPropagation()}>
          <div class="form-modal-handle"></div>
          <div class="form-modal-header">
            <span>{editingTask ? 'Edit Task' : 'New Task'}</span>
            <button class="form-modal-close" onclick={handleCloseEdit} aria-label="Close">
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
          {#key editingTask?.id}
            <TaskForm task={editingTask} onClose={handleCloseEdit} />
          {/key}
        </div>
      </div>
    {:else}
      <button class="fab" onclick={handleFabClick} aria-label="Add task">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
      </button>
    {/if}
  {/if}
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

  /* Floating Action Button */
  .fab {
    position: fixed;
    bottom: calc(1.5rem + env(safe-area-inset-bottom, 0));
    right: 1.5rem;
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: var(--accent-color);
    color: white;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
    z-index: 100;
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }

  .fab:hover {
    transform: scale(1.05);
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.3);
  }

  .fab:active {
    transform: scale(0.95);
  }

  .fab svg {
    width: 24px;
    height: 24px;
  }

  /* Mobile form modal */
  .form-modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 200;
    display: flex;
    align-items: flex-end;
    animation: fadeIn 0.2s ease;
  }

  .form-modal {
    width: 100%;
    background: var(--bg-primary);
    border-radius: 16px 16px 0 0;
    animation: slideUp 0.3s ease;
    max-height: 90vh;
    overflow-y: auto;
  }

  .form-modal-handle {
    width: 36px;
    height: 4px;
    background: var(--border-color);
    border-radius: 2px;
    margin: 8px auto;
  }

  .form-modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1rem 0.5rem;
    font-weight: 600;
  }

  .form-modal-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-secondary);
    border-radius: 50%;
  }

  .form-modal-close:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .form-modal-close svg {
    width: 20px;
    height: 20px;
  }

  /* Remove TaskForm border when in modal */
  .form-modal :global(.task-form) {
    border-top: none;
    padding-top: 0;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes slideUp {
    from { transform: translateY(100%); }
    to { transform: translateY(0); }
  }
</style>
