<script lang="ts">
  import { SegmentedControl } from '@skeletonlabs/skeleton-svelte';
  import { themeStore, AVAILABLE_THEMES, type ThemeMode, type ThemeName } from '$lib/stores/theme.svelte';

  function handleThemeChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    themeStore.setTheme(select.value as ThemeName);
  }

  function handleModeChange(details: { value: string | null }) {
    if (details.value) {
      themeStore.setMode(details.value as ThemeMode);
    }
  }
</script>

<div class="flex flex-col gap-6">
  <div class="flex justify-between items-center">
    <h2 class="m-0 text-xl font-semibold">Appearance</h2>
  </div>

  <div class="flex flex-col gap-4">
    <!-- Theme Selection -->
    <label class="flex flex-col gap-2">
      <span class="text-sm font-medium">Theme</span>
      <select
        class="input"
        value={themeStore.theme}
        onchange={handleThemeChange}
      >
        {#each AVAILABLE_THEMES as theme (theme.value)}
          <option value={theme.value}>{theme.label}</option>
        {/each}
      </select>
    </label>

    <!-- Mode Selection (Light/Dark/System) -->
    <div class="flex flex-col gap-2">
      <span class="text-sm font-medium">Mode</span>
      <SegmentedControl
        value={themeStore.mode}
        onValueChange={handleModeChange}
      >
        <SegmentedControl.Control class="w-full">
          <SegmentedControl.Indicator />
          <SegmentedControl.Item value="light">
            <SegmentedControl.ItemText>
              <span class="flex items-center gap-1.5">
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 7c-2.76 0-5 2.24-5 5s2.24 5 5 5 5-2.24 5-5-2.24-5-5-5zM2 13h2c.55 0 1-.45 1-1s-.45-1-1-1H2c-.55 0-1 .45-1 1s.45 1 1 1zm18 0h2c.55 0 1-.45 1-1s-.45-1-1-1h-2c-.55 0-1 .45-1 1s.45 1 1 1zM11 2v2c0 .55.45 1 1 1s1-.45 1-1V2c0-.55-.45-1-1-1s-1 .45-1 1zm0 18v2c0 .55.45 1 1 1s1-.45 1-1v-2c0-.55-.45-1-1-1s-1 .45-1 1zM5.99 4.58a.996.996 0 0 0-1.41 0 .996.996 0 0 0 0 1.41l1.06 1.06c.39.39 1.03.39 1.41 0s.39-1.03 0-1.41L5.99 4.58zm12.37 12.37a.996.996 0 0 0-1.41 0 .996.996 0 0 0 0 1.41l1.06 1.06c.39.39 1.03.39 1.41 0a.996.996 0 0 0 0-1.41l-1.06-1.06zm1.06-10.96a.996.996 0 0 0 0-1.41.996.996 0 0 0-1.41 0l-1.06 1.06c-.39.39-.39 1.03 0 1.41s1.03.39 1.41 0l1.06-1.06zM7.05 18.36a.996.996 0 0 0 0-1.41.996.996 0 0 0-1.41 0l-1.06 1.06c-.39.39-.39 1.03 0 1.41s1.03.39 1.41 0l1.06-1.06z"/>
                </svg>
                Light
              </span>
            </SegmentedControl.ItemText>
            <SegmentedControl.ItemHiddenInput />
          </SegmentedControl.Item>
          <SegmentedControl.Item value="dark">
            <SegmentedControl.ItemText>
              <span class="flex items-center gap-1.5">
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 3a9 9 0 1 0 9 9c0-.46-.04-.92-.1-1.36a5.389 5.389 0 0 1-4.4 2.26 5.403 5.403 0 0 1-3.14-9.8c-.44-.06-.9-.1-1.36-.1z"/>
                </svg>
                Dark
              </span>
            </SegmentedControl.ItemText>
            <SegmentedControl.ItemHiddenInput />
          </SegmentedControl.Item>
          <SegmentedControl.Item value="system">
            <SegmentedControl.ItemText>
              <span class="flex items-center gap-1.5">
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20 18c1.1 0 1.99-.9 1.99-2L22 6c0-1.1-.9-2-2-2H4c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2H0v2h24v-2h-4zM4 6h16v10H4V6z"/>
                </svg>
                System
              </span>
            </SegmentedControl.ItemText>
            <SegmentedControl.ItemHiddenInput />
          </SegmentedControl.Item>
        </SegmentedControl.Control>
      </SegmentedControl>
      <span class="text-xs text-surface-500">
        {#if themeStore.mode === 'system'}
          Currently using {themeStore.resolvedMode} mode based on system preference
        {/if}
      </span>
    </div>
  </div>
</div>
