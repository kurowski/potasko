<script lang="ts">
  import { untrack } from 'svelte';
  import { listStore } from "$lib/stores/lists.svelte";
  import { taskStore } from "$lib/stores/tasks.svelte";
  import type { Task } from "$lib/types";

  interface Props {
    task?: Task | null;
    onClose?: () => void;
  }

  let { task = null, onClose }: Props = $props();

  // Recurrence options (RRULE frequency values)
  const RECURRENCE_OPTIONS = [
    { value: "", label: "None" },
    { value: "FREQ=DAILY", label: "Daily" },
    { value: "FREQ=WEEKLY", label: "Weekly" },
    { value: "FREQ=MONTHLY", label: "Monthly" },
    { value: "FREQ=YEARLY", label: "Yearly" },
  ];

  // Form state - initialized from props (untrack captures initial value only)
  let title = $state(untrack(() => task?.title ?? ""));
  let description = $state(untrack(() => task?.description ?? ""));
  let dueDate = $state(untrack(() => task?.due_date ? task.due_date.split("T")[0] : ""));
  let priority = $state<string>(untrack(() => task?.priority?.toString() ?? ""));
  let rrule = $state(untrack(() => task?.rrule ?? ""));
  let saving = $state(false);

  const isEditing = $derived(task !== null);

  // Clear recurrence if due date is removed (recurrence requires a due date)
  $effect(() => {
    if (!dueDate && rrule) {
      rrule = "";
    }
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!title.trim() || !listStore.selectedListId) return;

    saving = true;
    try {
      const data = {
        title: title.trim(),
        description: description.trim() || null,
        due_date: dueDate ? new Date(dueDate).toISOString() : null,
        priority: priority ? parseInt(priority) : null,
        rrule: rrule || null,
      };

      if (isEditing && task) {
        await taskStore.update(task.id, data);
      } else {
        await taskStore.create({
          ...data,
          list_id: listStore.selectedListId,
        });
      }

      // Reset form
      title = "";
      description = "";
      dueDate = "";
      priority = "";
      rrule = "";
      onClose?.();
    } catch {
      // Error handled by store
    } finally {
      saving = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose?.();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<form class="task-form" onsubmit={handleSubmit}>
  <input
    type="text"
    class="title-input"
    bind:value={title}
    placeholder="Task title..."
    disabled={saving}
    autofocus
  />

  <textarea
    class="description-input"
    bind:value={description}
    placeholder="Description (optional)"
    rows="2"
    disabled={saving}
  ></textarea>

  <div class="form-row">
    <label class="form-field">
      <span>Due date</span>
      <input type="date" bind:value={dueDate} disabled={saving} />
    </label>

    <label class="form-field">
      <span>Repeat</span>
      <select
        bind:value={rrule}
        disabled={saving || !dueDate}
        title={!dueDate ? "Set a due date first" : ""}
      >
        {#each RECURRENCE_OPTIONS as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </label>

    <label class="form-field">
      <span>Priority</span>
      <div class="priority-select-wrapper">
        <select bind:value={priority} disabled={saving} class="priority-select">
          <option value="">None</option>
          <option value="1">High</option>
          <option value="5">Medium</option>
          <option value="9">Low</option>
        </select>
        {#if priority}
          <span
            class="priority-indicator priority-{priority === '1'
              ? 'high'
              : priority === '5'
                ? 'medium'
                : 'low'}"
          ></span>
        {/if}
      </div>
    </label>
  </div>

  <div class="form-actions">
    {#if isEditing}
      <button type="button" onclick={onClose} disabled={saving}>Cancel</button>
    {/if}
    <button type="submit" class="primary" disabled={!title.trim() || saving}>
      {saving ? "Saving..." : isEditing ? "Save" : "Add Task"}
    </button>
  </div>
</form>

<style>
  .task-form {
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--border-color);
    background: var(--bg-secondary);
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .title-input {
    font-size: 1rem;
    padding: 0.625rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--bg-primary);
  }

  .title-input:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .description-input {
    font-size: 0.875rem;
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    resize: vertical;
    font-family: inherit;
    background: var(--bg-primary);
  }

  .description-input:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .form-row {
    display: flex;
    gap: 1rem;
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    flex: 1;
  }

  .form-field span {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .form-field input,
  .form-field select {
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.875rem;
    background: var(--bg-primary);
  }

  .priority-select-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }

  .priority-select {
    flex: 1;
    padding-right: 2rem;
  }

  .priority-indicator {
    position: absolute;
    right: 1.75rem;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    pointer-events: none;
  }

  .priority-indicator.priority-high {
    background: var(--priority-high-text);
  }

  .priority-indicator.priority-medium {
    background: var(--priority-medium-text);
  }

  .priority-indicator.priority-low {
    background: var(--priority-low-text);
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }

  .form-actions button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .form-actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .form-actions button.primary {
    background: var(--accent-color);
    color: white;
  }

  .form-actions button:not(.primary) {
    background: transparent;
  }

  .form-actions button:not(.primary):hover {
    background: var(--bg-hover);
  }
</style>
