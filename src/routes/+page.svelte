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
  import {
    Header,
    HeaderUtilities,
    HeaderGlobalAction,
    SideNav,
    SideNavItems,
    Content,
    Breakpoint,
    ComposedModal,
    ModalHeader,
    ModalBody,
  } from 'carbon-components-svelte';
  import { Settings, Add, Close } from 'carbon-icons-svelte';

  let editingTask = $state<Task | null>(null);
  let showSettings = $state(false);
  let showTaskForm = $state(false);

  // Mobile responsiveness using Carbon's Breakpoint
  let breakpointSize: 'sm' | 'md' | 'lg' | 'xlg' | 'max' = $state('lg');
  const isMobile = $derived(breakpointSize === 'sm' || breakpointSize === 'md');
  let sidebarOpen = $state(false);

  // Get current view title for header
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

    // Setup sync event listeners to reload tasks when sync completes
    syncStore.setupEventListeners((listId) => {
      const view = listStore.selectedView;
      // Reload if viewing this list or a special view (Today/Overdue may include tasks from any list)
      if ((view?.type === 'list' && view.id === listId) || view?.type === 'special') {
        taskStore.reload();
      }
    });

    return () => {
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

  function handleCloseEdit() {
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

<Breakpoint bind:size={breakpointSize} />

<Header company="Potasko" platformName={isMobile ? currentViewTitle() : ''} bind:isSideNavOpen={sidebarOpen}>
  <HeaderUtilities>
    <HeaderGlobalAction
      aria-label="Settings"
      icon={Settings}
      isActive={showSettings}
      on:click={handleToggleSettings}
    />
  </HeaderUtilities>
</Header>

<SideNav bind:isOpen={sidebarOpen} rail={!isMobile}>
  <SideNavItems>
    <Sidebar
      {showSettings}
      onToggleSettings={handleToggleSettings}
      {isMobile}
      onClose={() => sidebarOpen = false}
    />
  </SideNavItems>
</SideNav>

<Content>
  {#if showSettings}
    <div class="settings-panel">
      <AccountList />
    </div>
  {:else if listStore.selectedView}
    <TaskListView onEditTask={handleEditTask} />
  {:else if listStore.loading}
    <div class="loading-state">Loading...</div>
  {:else}
    <div class="empty-state">
      <p>No lists yet</p>
      <p>Create a list to get started</p>
    </div>
  {/if}
</Content>

<!-- FAB for adding tasks -->
{#if canAddTasks && !showSettings && !showTaskForm}
  <button class="fab" onclick={handleAddTask} aria-label="Add task">
    <Add size={24} />
  </button>
{/if}

<!-- Task form modal (both desktop and mobile) -->
{#if showTaskForm}
  <ComposedModal open={true} on:close={handleCloseEdit} class="task-form-modal">
    <ModalHeader title={editingTask ? 'Edit Task' : 'New Task'} />
    <ModalBody>
      {#key editingTask?.id}
        <TaskForm task={editingTask} onClose={handleCloseEdit} />
      {/key}
    </ModalBody>
  </ComposedModal>
{/if}

<style>
  .loading-state,
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--cds-text-secondary, var(--text-secondary));
    min-height: 200px;
  }

  .empty-state p {
    margin: 0.25rem 0;
  }

  .settings-panel {
    padding: 1rem;
  }

  /* Floating Action Button - styled with Carbon tokens */
  .fab {
    position: fixed;
    bottom: calc(1.5rem + env(safe-area-inset-bottom, 0));
    right: 1.5rem;
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: var(--cds-interactive, var(--accent-color, #0f62fe));
    color: var(--cds-text-on-color, white);
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
    z-index: 8000;
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }

  .fab:hover {
    transform: scale(1.05);
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.3);
    background: var(--cds-interactive-hover, #0353e9);
  }

  .fab:active {
    transform: scale(0.95);
  }

  /* Remove TaskForm border when in modal */
  :global(.task-form-modal .bx--modal-content) {
    padding-top: 0;
  }
</style>
