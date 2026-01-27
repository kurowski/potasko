<script lang="ts">
  import { listStore } from '$lib/stores/lists.svelte';
  import { accountStore } from '$lib/stores/accounts.svelte';
  import type { TaskList } from '$lib/types';
  import {
    ComposedModal,
    ModalHeader,
    ModalBody,
    ModalFooter,
    TextInput,
    Select,
    SelectItem,
    Button,
    RadioButtonGroup,
    RadioButton,
  } from 'carbon-components-svelte';

  interface Props {
    list?: TaskList | null;
    onClose: () => void;
  }

  let { list = null, onClose }: Props = $props();

  // Predefined color palette
  const COLORS = [
    { value: '#6b7280', label: 'Gray' },
    { value: '#ef4444', label: 'Red' },
    { value: '#f97316', label: 'Orange' },
    { value: '#eab308', label: 'Yellow' },
    { value: '#22c55e', label: 'Green' },
    { value: '#14b8a6', label: 'Teal' },
    { value: '#3b82f6', label: 'Blue' },
    { value: '#8b5cf6', label: 'Purple' },
    { value: '#ec4899', label: 'Pink' },
  ];

  // Form state
  let name = $state(list?.name ?? '');
  let color = $state(list?.color ?? COLORS[0].value);
  let accountId = $state<number | null>(list?.account_id ?? null);
  let saving = $state(false);

  const isEditing = $derived(list !== null);

  async function handleSubmit() {
    if (!name.trim()) return;

    saving = true;
    try {
      if (isEditing && list) {
        await listStore.update(list.id, { name: name.trim(), color });
      } else {
        await listStore.create({
          name: name.trim(),
          color,
          account_id: accountId,
        });
      }
      onClose();
    } catch {
      // Error handled by store
    } finally {
      saving = false;
    }
  }
</script>

<ComposedModal open={true} on:close={onClose}>
  <ModalHeader title={isEditing ? 'Edit List' : 'New List'} />
  <ModalBody>
    <TextInput
      bind:value={name}
      labelText="List name"
      placeholder="Enter list name"
      disabled={saving}
      required
    />

    {#if !isEditing && accountStore.accounts.length > 0}
      <Select
        labelText="Account"
        bind:selected={accountId}
        disabled={saving}
      >
        <SelectItem value={null} text="Local only" />
        {#each accountStore.accounts as account (account.id)}
          <SelectItem value={account.id} text={account.name} />
        {/each}
      </Select>
    {/if}

    <div class="color-section">
      <span class="color-label">Color</span>
      <div class="color-grid">
        {#each COLORS as c}
          <button
            type="button"
            class="color-swatch"
            class:selected={color === c.value}
            style:background-color={c.value}
            onclick={() => color = c.value}
            title={c.label}
            disabled={saving}
          >
            {#if color === c.value}
              <svg viewBox="0 0 24 24" fill="white" class="check-icon">
                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
              </svg>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  </ModalBody>
  <ModalFooter>
    <Button kind="secondary" on:click={onClose} disabled={saving}>
      Cancel
    </Button>
    <Button on:click={handleSubmit} disabled={!name.trim() || saving}>
      {saving ? 'Saving...' : isEditing ? 'Save' : 'Create'}
    </Button>
  </ModalFooter>
</ComposedModal>

<style>
  .color-section {
    margin-top: 1rem;
  }

  .color-label {
    display: block;
    font-size: 0.75rem;
    font-weight: 400;
    color: var(--cds-text-secondary, #525252);
    margin-bottom: 0.5rem;
  }

  .color-grid {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .color-swatch {
    width: 32px;
    height: 32px;
    border: 2px solid transparent;
    border-radius: 4px;
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.1s ease, border-color 0.1s ease;
  }

  .color-swatch:hover:not(:disabled) {
    transform: scale(1.1);
  }

  .color-swatch.selected {
    border-color: var(--cds-text-primary, #161616);
  }

  .color-swatch:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .check-icon {
    width: 16px;
    height: 16px;
  }
</style>
