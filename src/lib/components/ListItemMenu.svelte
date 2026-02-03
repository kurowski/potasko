<script lang="ts">
  import type { TaskList } from '$lib/types';
  import Menu from '@smui/menu';
  import List, { Item, Text, Graphic } from '@smui/list';

  interface Props {
    onEdit?: (list: TaskList) => void;
    onDelete?: (list: TaskList) => void;
  }

  let { onEdit, onDelete }: Props = $props();

  let menu: Menu;
  let anchorElement: Element | undefined = $state();
  let open = $state(false);
  let currentList: TaskList | null = $state(null);

  export function openMenu(anchor: Element, list: TaskList) {
    anchorElement = anchor;
    currentList = list;
    open = true;
  }

  export function closeMenu() {
    open = false;
    currentList = null;
  }

  function handleEdit() {
    if (currentList && onEdit) {
      onEdit(currentList);
    }
    closeMenu();
  }

  function handleDelete() {
    if (currentList && onDelete) {
      onDelete(currentList);
    }
    closeMenu();
  }
</script>

<Menu bind:this={menu} bind:open anchor={false} {anchorElement} fixed>
  <List dense>
    <Item onclick={handleEdit}>
      <Graphic>
        <span class="material-icons">edit</span>
      </Graphic>
      <Text>Edit</Text>
    </Item>
    <Item onclick={handleDelete}>
      <Graphic>
        <span class="material-icons delete-menu-icon">delete</span>
      </Graphic>
      <Text>Delete</Text>
    </Item>
  </List>
</Menu>

<style>
  :global(.delete-menu-icon) {
    color: var(--mdc-theme-error, #dc2626);
  }
</style>
