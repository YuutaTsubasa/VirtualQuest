<script>
  // 一排切角選單按鈕。鍵盤／手把的焦點移動由 Controls.svelte 統一處理，這裡只負責一出現就把焦點放在第一顆。
  import { onMount } from 'svelte';
  import { play } from '../game/audio.js';
  import { ui } from '../ui.svelte.js';

  let { items, horizontal = false, width = '40cqw' } = $props();
  let nav;

  onMount(() => {
    nav.querySelector('button')?.focus({ preventScroll: true });
  });

  function choose(item) {
    play('select');
    item.action();
  }
</script>

<nav class="menu" class:row={horizontal} bind:this={nav} style="--menu-width: {width}">
  {#each items as item (item.label)}
    <button
      type="button"
      class="vbtn arrow"
      class:gold={item.primary}
      onfocus={() => (ui.input === 'key' || ui.input === 'pad') && play('move')}
      onclick={() => choose(item)}
    >
      {item.label}
    </button>
  {/each}
</nav>

<style>
  .menu {
    display: flex;
    flex-direction: column;
    gap: 1.6cqh;
    width: var(--menu-width);
  }
  .menu.row {
    flex-direction: row;
    width: auto;
  }
  .menu.row .vbtn {
    padding-left: 6.4cqh;
    padding-right: 5cqh;
  }
</style>
