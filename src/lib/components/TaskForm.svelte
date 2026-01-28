<script lang="ts">
  import { untrack } from 'svelte';
  import { listStore } from "$lib/stores/lists.svelte";
  import { taskStore } from "$lib/stores/tasks.svelte";
  import type { Task } from "$lib/types";
  import {
    Form,
    TextInput,
    TextArea,
    DatePicker,
    DatePickerInput,
    Select,
    SelectItem,
    Button,
    ButtonSet,
  } from 'carbon-components-svelte';

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

  function handleDateChange(e: CustomEvent) {
    const dateStr = e.detail.dateStr;
    if (dateStr) {
      // Carbon DatePicker returns MM/DD/YYYY format, convert to YYYY-MM-DD
      const [month, day, year] = dateStr.split('/');
      dueDate = `${year}-${month.padStart(2, '0')}-${day.padStart(2, '0')}`;
    } else {
      dueDate = "";
    }
  }

  // Convert YYYY-MM-DD to MM/DD/YYYY for Carbon DatePicker display
  const displayDate = $derived(() => {
    if (!dueDate) return "";
    const [year, month, day] = dueDate.split('-');
    return `${month}/${day}/${year}`;
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<Form on:submit={handleSubmit} class="task-form">
  <TextInput
    bind:value={title}
    labelText="Task title"
    placeholder="What needs to be done?"
    disabled={saving}
    required
  />

  <TextArea
    bind:value={description}
    labelText="Description"
    placeholder="Add details (optional)"
    rows={2}
    disabled={saving}
  />

  <div class="form-row">
    <DatePicker
      datePickerType="single"
      dateFormat="m/d/Y"
      value={displayDate()}
      on:change={handleDateChange}
    >
      <DatePickerInput
        labelText="Due date"
        placeholder="mm/dd/yyyy"
        disabled={saving}
      />
    </DatePicker>

    <Select
      labelText="Repeat"
      bind:selected={rrule}
      disabled={saving || !dueDate}
    >
      {#each RECURRENCE_OPTIONS as opt}
        <SelectItem value={opt.value} text={opt.label} />
      {/each}
    </Select>

    <Select
      labelText="Priority"
      bind:selected={priority}
      disabled={saving}
    >
      <SelectItem value="" text="None" />
      <SelectItem value="1" text="High" />
      <SelectItem value="5" text="Medium" />
      <SelectItem value="9" text="Low" />
    </Select>
  </div>

  <ButtonSet class="form-actions">
    {#if isEditing}
      <Button kind="secondary" on:click={onClose} disabled={saving}>
        Cancel
      </Button>
    {/if}
    <Button type="submit" disabled={!title.trim() || saving}>
      {saving ? "Saving..." : isEditing ? "Save" : "Add Task"}
    </Button>
  </ButtonSet>
</Form>

<style>
  :global(.task-form) {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .form-row {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .form-row > :global(*) {
    flex: 1;
    min-width: 120px;
  }

  :global(.form-actions) {
    justify-content: flex-end;
  }

  /* Mobile adjustments */
  @media (max-width: 1056px) {
    :global(.task-form) {
      padding-bottom: calc(1rem + env(safe-area-inset-bottom, 0));
    }

    .form-row {
      flex-direction: column;
    }

    .form-row > :global(*) {
      min-width: 100%;
    }
  }
</style>
