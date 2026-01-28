<script lang="ts">
  import { onMount } from 'svelte';
  import type { Task } from '$lib/types';
  import { listStore } from '$lib/stores/lists.svelte';
  import { taskStore } from '$lib/stores/tasks.svelte';
  import { syncStore } from '$lib/stores/sync.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import TaskListView from '$lib/components/TaskListView.svelte';
  import TaskForm from '$lib/components/TaskForm.svelte';
  import ThemeSettings from '$lib/components/ThemeSettings.svelte';
  import AccountList from '$lib/components/AccountList.svelte';

  let editingTask = $state<Task | null>(null);
  let showSettings = $state(false);
  let showTaskForm = $state(false);

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
    themeStore.init();
    listStore.load();
    checkMobile();
    window.addEventListener('resize', checkMobile);

    // Setup sync event listeners to reload tasks when sync completes
    syncStore.setupEventListeners((listId) => {
      const view = listStore.selectedView;
      // Reload if viewing this list or a special view (Today/Overdue may include tasks from any list)
      if ((view?.type === 'list' && view.id === listId) || view?.type === 'special') {
        taskStore.reload();
      }
    });

    return () => {
      window.removeEventListener('resize', checkMobile);
      syncStore.cleanup();
    };
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
    showTaskForm = true;
  }

  function handleCloseForm() {
    editingTask = null;
    showTaskForm = false;
  }

  function handleToggleSettings() {
    showSettings = !showSettings;
  }

  function handleAddTask() {
    editingTask = null;
    showTaskForm = true;
  }
</script>

<div class="h-full flex flex-col md:flex-row overflow-hidden">
  <!-- Mobile header -->
  {#if isMobile}
    <header class="flex items-center gap-3 px-4 py-3 pt-[calc(0.75rem+var(--safe-area-top))] bg-surface-100-900 border-b border-surface-300-700">
      <button
        class="btn-icon preset-tonal"
        onclick={toggleSidebar}
        aria-label="Toggle menu"
      >
        <svg class="w-6 h-6" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="3" y1="12" x2="21" y2="12"></line>
          <line x1="3" y1="6" x2="21" y2="6"></line>
          <line x1="3" y1="18" x2="21" y2="18"></line>
        </svg>
      </button>
      <h1 class="m-0 text-lg font-semibold">{currentViewTitle()}</h1>
    </header>
  {/if}

  <Sidebar
    {showSettings}
    onToggleSettings={handleToggleSettings}
    {isMobile}
    isOpen={sidebarOpen}
    onClose={closeSidebar}
  />

  <!-- Mobile backdrop -->
  {#if isMobile && sidebarOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div
      class="fixed inset-0 bg-black/50 z-[99] transition-opacity"
      onclick={closeSidebar}
    ></div>
  {/if}

  <main class="flex-1 flex flex-col min-w-0 min-h-0">
    {#if showSettings}
      <div class="flex-1 p-6 md:p-8 overflow-y-auto bg-surface-50-950">
        <div class="max-w-2xl mx-auto flex flex-col gap-8">
          <ThemeSettings />
          <hr class="border-surface-300-700" />
          <AccountList />
        </div>
      </div>
    {:else if listStore.selectedView}
      <TaskListView onEditTask={handleEditTask} onAddTask={handleAddTask} />
    {:else if listStore.loading}
      <div class="flex-1 flex flex-col items-center justify-center text-surface-500">Loading...</div>
    {:else}
      <div class="flex-1 flex flex-col items-center justify-center text-surface-500">
        <p class="my-1">No lists yet</p>
        <p class="my-1">Create a list to get started</p>
      </div>
    {/if}
  </main>

  <!-- Task form modal (both mobile and desktop) -->
  {#if showTaskForm && canAddTasks && !showSettings}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div
      class="fixed inset-0 bg-black/50 z-[200] flex animate-fadeIn
             {isMobile ? 'items-end' : 'items-center justify-center p-4'}"
      onclick={handleCloseForm}
    >
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div
        class="bg-surface-50-950 overflow-y-auto
               {isMobile
                 ? 'w-full rounded-t-2xl animate-slideUp max-h-[90vh]'
                 : 'w-full max-w-lg rounded-xl shadow-xl animate-scaleIn'}"
        onclick={(e) => e.stopPropagation()}
      >
        {#if isMobile}
          <div class="w-9 h-1 bg-surface-300-700 rounded-full mx-auto mt-2"></div>
        {/if}
        <div class="flex items-center justify-between px-4 py-3 {isMobile ? '' : 'border-b border-surface-300-700'}">
          <span class="font-semibold text-lg">{editingTask ? 'Edit Task' : 'New Task'}</span>
          <button
            class="btn-icon preset-tonal"
            onclick={handleCloseForm}
            aria-label="Close"
          >
            <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
        {#key editingTask?.id}
          <TaskForm task={editingTask} onClose={handleCloseForm} />
        {/key}
      </div>
    </div>
  {/if}

  <!-- Mobile FAB -->
  {#if isMobile && canAddTasks && !showSettings && !showTaskForm}
    <button
      class="fixed bottom-[calc(1.5rem+var(--safe-area-bottom))] right-6 w-14 h-14 rounded-full
             bg-primary-500 text-white border-none cursor-pointer
             flex items-center justify-center shadow-lg z-[100]
             transition-transform hover:scale-105 active:scale-95"
      onclick={handleAddTask}
      aria-label="Add task"
    >
      <svg class="w-6 h-6" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <line x1="12" y1="5" x2="12" y2="19"></line>
        <line x1="5" y1="12" x2="19" y2="12"></line>
      </svg>
    </button>
  {/if}
</div>

<style>
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes slideUp {
    from { transform: translateY(100%); }
    to { transform: translateY(0); }
  }

  @keyframes scaleIn {
    from { opacity: 0; transform: scale(0.95); }
    to { opacity: 1; transform: scale(1); }
  }

  .animate-fadeIn {
    animation: fadeIn 0.2s ease;
  }

  .animate-slideUp {
    animation: slideUp 0.3s ease;
  }

  .animate-scaleIn {
    animation: scaleIn 0.2s ease;
  }
</style>
