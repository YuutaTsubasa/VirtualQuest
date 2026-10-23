<script>
  // 角色頭像格子：兩排，滑鼠停留／鍵盤手把焦點會移動游標；觸控第一下預覽、第二下確定
  import { asset } from '../art-style.svelte.js';
  import { CHARACTERS, isUnlocked } from '../game/characters.js';
  import { play } from '../game/audio.js';
  import { ui } from '../ui.svelte.js';
  import { t } from '../i18n.svelte.js';

  // cursor：目前看的角色（索引）；selected：已選定的角色 id（打勾）；tags：{ id: ['1P'] } 角標
  let { cursor = $bindable(0), selected = null, tags = {}, showLocked = true, onpick = () => {} } = $props();

  const list = $derived(CHARACTERS.filter((c) => showLocked || isUnlocked(c)));
  const columns = $derived(Math.ceil(list.length / 2));
  let root;

  export function focusIndex(i) {
    const id = CHARACTERS[i]?.id;
    root?.querySelector(`[data-id="${id}"]`)?.focus({ preventScroll: true });
  }

  function look(c) {
    cursor = CHARACTERS.indexOf(c);
  }

  function tap(c) {
    if (!isUnlocked(c)) return play('deny');
    if (ui.input === 'touch' && CHARACTERS[cursor] !== c) {
      look(c);
      return;
    }
    look(c);
    onpick(c);
  }
</script>

<div class="grid" bind:this={root} style="--cols: {columns}">
  {#each list as c (c.id)}
    {@const open = isUnlocked(c)}
    <button
      type="button"
      class="card"
      class:selected={selected === c.id}
      class:looking={CHARACTERS[cursor] === c}
      class:locked={!open}
      data-id={c.id}
      onmouseenter={() => ui.input === 'pointer' && look(c)}
      onfocus={() => {
        look(c);
        if (ui.input === 'key' || ui.input === 'pad') play('move');
      }}
      onclick={() => tap(c)}
    >
      {#if open}
        <img src={c.portraits.normal} alt="" draggable="false" />
      {:else}
        <span class="lock"><img src="{asset('ui/icons/lock.webp')}" alt="" /></span>
      {/if}
      <span>{open ? c.name : t('？？？')}</span>
      {#if tags[c.id]?.length}<span class="tags">{#each tags[c.id] as tag, i (i)}<i class="tag">{tag}</i>{/each}</span>{/if}
    </button>
  {/each}
</div>

<style>
  .grid {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(var(--cols), 1fr);
    gap: 1.4cqh;
    align-content: center;
  }
  .card.looking:not(.selected) {
    border-color: var(--gold-deep);
  }
  .lock {
    flex: 1;
    display: grid;
    place-items: center;
    background: linear-gradient(#3b5a9a, #233c75);
  }
  .lock img {
    width: 55%;
    flex: none;
  }
</style>
