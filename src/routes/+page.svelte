<script lang="ts">
  import { onMount } from 'svelte';
  import type { Task } from '$lib/types';
  import { listStore } from '$lib/stores/lists.svelte';
  import { taskStore } from '$lib/stores/tasks.svelte';
  import { syncStore } from '$lib/stores/sync.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import TaskListView from '$lib/components/TaskListView.svelte';
  import TaskForm from '$lib/components/TaskForm.svelte';
  import AccountList from '$lib/components/AccountList.svelte';
  import TopAppBar, { Row, Section, Title } from '@smui/top-app-bar';
  import IconButton from '@smui/icon-button';
  import Fab from '@smui/fab';
  import Dialog, { Title as DialogTitle, Content, Actions } from '@smui/dialog';
  import Button from '@smui/button';
  import CircularProgress from '@smui/circular-progress';

  let editingTask = $state<Task | null>(null);
  let showSettings = $state(false);
  let taskDialogOpen = $state(false);

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
    taskDialogOpen = true;
  }

  function handleAddTask() {
    editingTask = null;
    taskDialogOpen = true;
  }

  function handleCloseTaskDialog() {
    taskDialogOpen = false;
    editingTask = null;
  }

  function handleToggleSettings() {
    showSettings = !showSettings;
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="app-layout" class:mobile={isMobile}>
  {#if isMobile}
    <TopAppBar variant="fixed" dense class="mobile-app-bar">
      <Row>
        <Section>
          <IconButton onclick={toggleSidebar}>
            <span class="material-icons">menu</span>
          </IconButton>
          <Title>{currentViewTitle()}</Title>
        </Section>
      </Row>
    </TopAppBar>
    <div class="top-app-bar-spacer"></div>
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
      <TaskListView onEditTask={handleEditTask} onAddTask={handleAddTask} />
    {:else if listStore.loading}
      <div class="loading-state">
        <CircularProgress indeterminate />
        <p>Loading...</p>
      </div>
    {:else}
      <div class="empty-state">
        <span class="material-icons empty-icon">checklist</span>
        <p>No lists yet</p>
        <p class="hint">Create a list to get started</p>
      </div>
    {/if}
  </main>

  <!-- FAB for mobile - add task -->
  {#if isMobile && canAddTasks && !showSettings}
    <Fab
      onclick={handleAddTask}
      class="add-task-fab"
      extended
    >
      <span class="material-icons">add</span>
      Add Task
    </Fab>
  {/if}

  <!-- Task Dialog (used for both desktop and mobile) -->
  <Dialog
    bind:open={taskDialogOpen}
    fullscreen={isMobile}
    aria-labelledby="task-dialog-title"
    surface$style={isMobile ? '' : 'max-width: 500px; width: 90vw;'}
  >
    <DialogTitle id="task-dialog-title">
      {editingTask ? 'Edit Task' : 'New Task'}
    </DialogTitle>
    <Content>
      {#key editingTask?.id}
        <TaskForm task={editingTask} onClose={handleCloseTaskDialog} />
      {/key}
    </Content>
  </Dialog>
</div>

<style>
  .app-layout {
    display: flex;
    height: 100%;
    overflow: hidden;
    background: var(--mdc-theme-background, #f9fafb);
  }

  .app-layout.mobile {
    flex-direction: column;
  }

  .top-app-bar-spacer {
    height: 48px; /* Dense app bar height */
    flex-shrink: 0;
  }

  :global(.mobile-app-bar) {
    position: fixed !important;
    top: 0;
    left: 0;
    right: 0;
    z-index: var(--app-z-app-bar);
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    background: var(--mdc-theme-surface, #fff);
  }

  .loading-state,
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--mdc-theme-text-secondary-on-background, rgba(0, 0, 0, 0.6));
    gap: var(--app-spacing-2);
  }

  .empty-icon {
    font-size: 64px;
    opacity: 0.4;
    margin-bottom: var(--app-spacing-2);
  }

  .empty-state p {
    margin: var(--app-spacing-1) 0;
  }

  .empty-state .hint {
    font-size: var(--app-font-size-sm);
  }

  .settings-panel {
    flex: 1;
    padding: var(--app-spacing-6);
    overflow-y: auto;
  }

  @media (max-width: 768px) {
    .settings-panel {
      padding: var(--app-spacing-4);
    }
  }

  /* Backdrop for mobile sidebar */
  .backdrop {
    position: fixed;
    inset: 0;
    background: var(--app-color-backdrop);
    z-index: var(--app-z-backdrop);
    opacity: 0;
    pointer-events: none;
    transition: opacity var(--app-transition-slow);
  }

  .backdrop.visible {
    opacity: 1;
    pointer-events: auto;
  }

  /* FAB positioning */
  :global(.add-task-fab) {
    position: fixed !important;
    bottom: calc(var(--app-spacing-6) + env(safe-area-inset-bottom, 0));
    right: var(--app-spacing-6);
    z-index: var(--app-z-app-bar);
  }

  /* Dialog content adjustments */
  :global(.mdc-dialog .task-form) {
    padding: 0;
  }

  :global(.mdc-dialog .form-actions) {
    padding-top: var(--app-spacing-4);
    padding-bottom: 0;
  }
</style>
