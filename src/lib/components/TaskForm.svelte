<script lang="ts">
  import { untrack } from 'svelte';
  import { listStore } from "$lib/stores/lists.svelte";
  import { taskStore } from "$lib/stores/tasks.svelte";
  import type { Task } from "$lib/types";
  import Textfield from '@smui/textfield';
  import Select, { Option } from '@smui/select';
  import Button from '@smui/button';

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

  const PRIORITY_OPTIONS = [
    { value: "", label: "None" },
    { value: "1", label: "High" },
    { value: "5", label: "Medium" },
    { value: "9", label: "Low" },
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
  <Textfield
    variant="outlined"
    bind:value={title}
    label="Task title"
    disabled={saving}
    style="width: 100%;"
  />

  <Textfield
    variant="outlined"
    textarea
    bind:value={description}
    label="Description (optional)"
    disabled={saving}
    style="width: 100%;"
  />

  <div class="form-row">
    <div class="form-field">
      <Textfield
        variant="outlined"
        type="date"
        bind:value={dueDate}
        label="Due date"
        disabled={saving}
        style="width: 100%;"
      />
    </div>

    <div class="form-field">
      <Select
        variant="outlined"
        bind:value={rrule}
        label="Repeat"
        disabled={saving || !dueDate}
        style="width: 100%;"
      >
        {#each RECURRENCE_OPTIONS as opt}
          <Option value={opt.value}>{opt.label}</Option>
        {/each}
      </Select>
    </div>

    <div class="form-field">
      <div class="priority-wrapper">
        <Select
          variant="outlined"
          bind:value={priority}
          label="Priority"
          disabled={saving}
          style="width: 100%;"
        >
          {#each PRIORITY_OPTIONS as opt}
            <Option value={opt.value}>{opt.label}</Option>
          {/each}
        </Select>
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
    </div>
  </div>

  <div class="form-actions">
    {#if isEditing}
      <Button variant="text" onclick={onClose} disabled={saving}>
        Cancel
      </Button>
    {/if}
    <Button variant="raised" type="submit" disabled={!title.trim() || saving}>
      {saving ? "Saving..." : isEditing ? "Save" : "Add Task"}
    </Button>
  </div>
</form>

<style>
  .task-form {
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .form-row {
    display: flex;
    gap: 1rem;
  }

  .form-field {
    flex: 1;
    min-width: 0;
  }

  .priority-wrapper {
    position: relative;
  }

  .priority-indicator {
    position: absolute;
    right: 2.5rem;
    top: 50%;
    transform: translateY(-50%);
    width: 10px;
    height: 10px;
    border-radius: 50%;
    pointer-events: none;
  }

  .priority-indicator.priority-high {
    background: #dc2626;
  }

  .priority-indicator.priority-medium {
    background: #f59e0b;
  }

  .priority-indicator.priority-low {
    background: #3b82f6;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    padding-top: 0.5rem;
  }

  /* Mobile adjustments */
  @media (max-width: 768px) {
    .task-form {
      padding: 1rem;
      padding-bottom: calc(1rem + env(safe-area-inset-bottom, 0));
    }

    .form-row {
      flex-direction: column;
      gap: 1rem;
    }
  }
</style>
