<script>
  import '../app.css';
  import 'carbon-components-svelte/css/all.css';
  import { Theme } from 'carbon-components-svelte';

  let { children } = $props();

  let theme = $state('white');

  $effect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    theme = mediaQuery.matches ? 'g90' : 'white';

    const handler = (e) => {
      theme = e.matches ? 'g90' : 'white';
    };
    mediaQuery.addEventListener('change', handler);

    return () => mediaQuery.removeEventListener('change', handler);
  });
</script>

<Theme bind:theme />

{@render children()}
