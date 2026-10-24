<script>
  // 單機遊玩：選模式（一人闖關／多人同樂／電腦示範），右邊是預覽
  import { asset } from '../art-style.svelte.js';
  import { onMount } from 'svelte';
  import { CHARACTERS } from '../game/characters.js';
  import { play } from '../game/audio.js';
  import { ui } from '../ui.svelte.js';
  import ScreenFrame from './ScreenFrame.svelte';
  import SpritePose from './SpritePose.svelte';
  import WorldScene from './WorldScene.svelte';
  import BackButton from './BackButton.svelte';
  import ControlHint from './ControlHint.svelte';
  import { t } from '../i18n.svelte.js';

  let { onpick, onback } = $props();

  const MODES = [
    { id: 'solo', title: '一人闖關', sub: '帶著電腦隊友出發', tag: '1P＋CPU', tags: ['1P', 'CPU'], caption: '電腦隊友最多三位！', world: 0 },
    { id: 'party', title: '多人同樂', sub: '同一台電腦，最多四人', tag: '1P～4P', tags: ['1P', '2P'], caption: '鍵盤兩人，手把最多四人！', world: 1 },
    { id: 'watch', title: '電腦示範', sub: '看電腦隊伍闖關', tag: 'CPU', tags: ['CPU', 'CPU'], caption: '看看電腦怎麼打！', world: 2 },
  ];

  let current = $state(0);
  const mode = $derived(MODES[current]);
  // 有模式預覽插圖（ui/mode-*）就用插圖，沒有就用關卡背景＋兩位角色的動作圖
  const modeArt = $derived(asset(`ui/mode-${mode.id}.webp`));
  const watchIcon = $derived(asset('ui/icons/watch.webp') ?? asset('ui/icons/watch.webp', 'anime90'));

  // 預覽用的兩位角色：每次進來隨機挑
  const pool = CHARACTERS.filter((c) => !c.hidden);
  const pickTwo = () => {
    const a = pool[Math.floor(Math.random() * pool.length)];
    let b = pool[Math.floor(Math.random() * pool.length)];
    if (b === a) b = pool[(pool.indexOf(a) + 4) % pool.length];
    return [a, b];
  };
  const pairs = MODES.map(pickTwo);

  let list;
  onMount(() => list.querySelector('button')?.focus({ preventScroll: true }));

  function choose(i) {
    play('select');
    onpick(MODES[i].id);
  }
</script>

{#snippet icon(id)}
  {#if id === 'solo'}
    <img src="{asset('ui/icons/gamepad.webp')}" alt="" />
  {:else if id === 'party'}
    <span class="pair"><img src="{asset('ui/icons/gamepad.webp')}" alt="" /><img src="{asset('ui/icons/gamepad.webp')}" alt="" /></span>
  {:else}
    <img src={watchIcon} alt="" />
  {/if}
{/snippet}

<ScreenFrame title={t('單機遊玩')} subtitle={t('選擇遊玩模式')}>
  <div class="modes" bind:this={list}>
    {#each MODES as m, i (m.id)}
      <button
        type="button"
        class="mode vbtn plain"
        class:active={i === current}
        onfocus={() => {
          current = i;
          if (ui.input === 'key' || ui.input === 'pad') play('move');
        }}
        onmouseenter={() => ui.input === 'pointer' && (current = i)}
        onclick={() => (ui.input === 'touch' && current !== i ? (current = i) : choose(i))}
      >
        <span class="icon">{@render icon(m.id)}</span>
        <span class="text"><b>{t(m.title)}</b><small>{t(m.sub)}</small></span>
        <span class="tag">{m.tag}</span>
      </button>
    {/each}
  </div>

  <div class="preview panel">
    <div class="stage-art">
      {#key current}
        {#if modeArt}
          <img class="mode-art" src={modeArt} alt="" />
        {:else}
          <WorldScene world={mode.world} />
          <div class="side left">
            <span class="who">{mode.tags[0]}</span>
            <SpritePose character={pairs[current][0]} pose="spike" size="32cqh" />
          </div>
          <div class="side right">
            <span class="who cpu" class:p2={mode.tags[1] === '2P'}>{mode.tags[1]}</span>
            <SpritePose character={pairs[current][1]} pose="run1" size="32cqh" />
          </div>
        {/if}
      {/key}
    </div>
    <div class="caption">{t(mode.caption)}</div>
  </div>

  {#snippet foot()}
    <BackButton onclick={onback} />
    <span></span>
    <ControlHint />
  {/snippet}
</ScreenFrame>

<style>
  .modes {
    flex: 1.15;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2.2cqh;
  }
  .mode {
    --cut: 2.4cqh;
    justify-content: flex-start;
    min-height: 15cqh;
    padding: 1cqh 6cqh 1cqh 5cqh;
    letter-spacing: 0;
    text-indent: 0;
    gap: 3cqh;
    text-align: left;
  }
  /* 目前選的模式：朱紅（青綠留給次要按鈕） */
  .mode.active {
    --face: var(--accent);
    --fg: var(--on-dark);
  }
  .mode.active .tag {
    border-left-color: var(--on-dark);
  }
  .icon {
    width: 13cqh;
    display: flex;
    justify-content: center;
  }
  .icon img {
    width: 11cqh;
  }

  .pair {
    display: flex;
  }
  .pair img {
    width: 7.5cqh;
  }
  .pair img + img {
    margin-left: -1.6cqh;
  }
  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.6cqh;
  }
  .text b {
    font-size: 5.6cqh;
    letter-spacing: 0.1em;
    line-height: 1;
  }
  .text small {
    font-size: 2.4cqh;
    font-weight: 800;
  }
  .tag {
    font-size: 2.6cqh;
    font-weight: 900;
    border-left: 0.35cqh solid var(--edge);
    padding-left: 2.4cqh;
    white-space: nowrap;
  }

  .preview {
    flex: 1;
    padding: 1.2cqh;
    gap: 1.2cqh;
  }
  .stage-art {
    position: relative;
    flex: 1;
    border: 0.35cqh solid var(--edge);
    background-position: center 70%;
    background-size: cover;
    overflow: hidden;
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    padding: 0 3cqh 1cqh;
  }
  .side {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    animation: pop 0.25s ease-out;
  }
  .who {
    background: var(--ink-fill);
    color: var(--on-dark);
    font-weight: 900;
    font-size: 3cqh;
    padding: 0.2cqh 3cqh;
    border: 0.35cqh solid var(--cream);
    border-radius: 1cqh;
    margin-bottom: -1cqh;
    z-index: 1;
  }
  .who.cpu {
    background: var(--teal);
  }
  .who.p2 {
    background: var(--accent);
  }


  .mode-art {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    animation: pop 0.25s ease-out;
  }
  .caption {
    background: var(--cream);
    border: 0.35cqh solid var(--edge);
    text-align: center;
    font-weight: 900;
    font-size: 3.2cqh;
    letter-spacing: 0.15em;
    padding: 0.8cqh;
  }

  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(1.6cqh);
    }
  }
</style>
