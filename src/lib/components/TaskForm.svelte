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

  function getPriorityDotClass(p: string): string {
    if (p === '1') return 'bg-error-500';
    if (p === '5') return 'bg-warning-500';
    if (p === '9') return 'bg-primary-500';
    return '';
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<form
  class="p-4 md:px-6 flex flex-col gap-3 pb-[calc(1rem+var(--safe-area-bottom))]"
  onsubmit={handleSubmit}
>
  <input
    type="text"
    class="input"
    bind:value={title}
    placeholder="Task title..."
    disabled={saving}
    autofocus
  />

  <textarea
    class="input resize-y"
    bind:value={description}
    placeholder="Description (optional)"
    rows="2"
    disabled={saving}
  ></textarea>

  <div class="flex flex-col md:flex-row gap-3">
    <label class="flex flex-col gap-1 flex-1">
      <span class="text-xs text-surface-500">Due date</span>
      <input
        type="date"
        class="input min-h-11"
        bind:value={dueDate}
        disabled={saving}
      />
    </label>

    <label class="flex flex-col gap-1 flex-1">
      <span class="text-xs text-surface-500">Repeat</span>
      <select
        class="input min-h-11"
        bind:value={rrule}
        disabled={saving || !dueDate}
        title={!dueDate ? "Set a due date first" : ""}
      >
        {#each RECURRENCE_OPTIONS as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </label>

    <label class="flex flex-col gap-1 flex-1">
      <span class="text-xs text-surface-500">Priority</span>
      <div class="relative flex items-center">
        <select class="input min-h-11 pr-8 flex-1" bind:value={priority} disabled={saving}>
          <option value="">None</option>
          <option value="1">High</option>
          <option value="5">Medium</option>
          <option value="9">Low</option>
        </select>
        {#if priority}
          <span
            class="absolute right-7 w-2 h-2 rounded-full pointer-events-none {getPriorityDotClass(priority)}"
          ></span>
        {/if}
      </div>
    </label>
  </div>

  <div class="flex justify-end gap-2">
    <button
      type="button"
      class="btn preset-outlined"
      onclick={onClose}
      disabled={saving}
    >
      Cancel
    </button>
    <button
      type="submit"
      class="btn preset-filled-primary-500"
      disabled={!title.trim() || saving}
    >
      {saving ? "Saving..." : isEditing ? "Save" : "Add Task"}
    </button>
  </div>
</form>
