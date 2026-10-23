<script>
  // 單機選角（art-work/ui-concept-v1/02-character-select）：左邊角色名冊、中間目前角色的大頭像、
  // 右邊能力與闖關設定，底下是同一隊的四個席位。依序選每位真人玩家的角色（電腦隊友隨機）。
  import { onMount, untrack } from 'svelte';
  import { CHARACTERS, isUnlocked } from '../game/characters.js';
  import { WORLDS } from '../game/stages.js';
  import { SLOT_COLORS } from '../game/constants.js';
  import { reachedWorld } from '../game/progress.js';
  import { play } from '../game/audio.js';
  import { ui } from '../ui.svelte.js';
  import ScreenFrame from './ScreenFrame.svelte';
  import BackButton from './BackButton.svelte';
  import ControlHint from './ControlHint.svelte';
  import CharacterGrid from './CharacterGrid.svelte';
  import Stepper from './Stepper.svelte';
  import { asset } from '../art-style.svelte.js';
  import { t } from '../i18n.svelte.js';

  // onstart({ characters, humans, settings })、onback()
  let { mode, onstart, onback } = $props();

  // 只看進來時的模式（換模式會重新建這個畫面）
  const start = untrack(() => mode);
  // 真人人數：一人闖關 1、多人同樂 2～4、電腦示範 0
  let humans = $state(start === 'party' ? 2 : start === 'solo' ? 1 : 0);
  let allies = $state(start === 'watch' ? 4 : start === 'solo' ? 1 : 0);
  let difficulty = $state('normal');
  let world = $state(0);
  let lives = $state(3);
  const maxWorld = reachedWorld();

  let picks = $state([]);
  let cursor = $state(4); // 預設小春
  let grid;
  let startButton = $state();

  const current = $derived(CHARACTERS[cursor]);
  const open = $derived(isUnlocked(current));
  const maxAllies = $derived(Math.max(start === 'watch' ? 1 : 0, 4 - humans));
  $effect(() => {
    if (allies > maxAllies) allies = maxAllies;
    if (picks.length > humans) picks.length = humans;
  });
  // 正在選第幾位真人（全部選完 = humans）
  const step = $derived(Math.min(picks.length, humans));
  const done = $derived(step >= humans);
  const tags = $derived(picks.reduce((all, c, i) => ({ ...all, [c.id]: [...(all[c.id] ?? []), `${i + 1}P`] }), {}));

  const subtitle = $derived(done ? t('調好設定就出發吧！') : humans > 1 ? t('{n}P 選擇角色', { n: step + 1 }) : t('選擇角色'));

  const STATS = [
    ['速度', 'speed'],
    ['力量', 'power'],
    ['體力', 'hp'],
  ];
  const humanOptions = [2, 3, 4].map((n) => ({ value: n, label: t('{n} 人', { n }) }));
  const allyOptions = $derived(
    Array.from({ length: maxAllies + 1 }, (_, i) => i)
      .filter((n) => start !== 'watch' || n > 0)
      .map((n) => ({ value: n, label: t('{n} 位', { n }) })),
  );
  const difficultyOptions = [
    { value: 'easy', label: t('簡單') },
    { value: 'normal', label: t('普通') },
    { value: 'hard', label: t('困難') },
  ];
  const worldOptions = WORLDS.slice(0, maxWorld + 1).map((w, i) => ({ value: i, label: t(w.name) }));
  const livesOptions = [1, 3, 5].map((n) => ({ value: n, label: String(n) }));

  onMount(() => grid.focusIndex(cursor));

  function pick(character = current) {
    if (done) return;
    if (!isUnlocked(character)) return play('deny');
    play('pick');
    picks.push(character);
    if (picks.length >= humans) {
      if (ui.input !== 'touch') startButton?.focus({ preventScroll: true });
      return;
    }
    let next = CHARACTERS.indexOf(character);
    do next = (next + 1) % CHARACTERS.length;
    while (!isUnlocked(CHARACTERS[next]));
    cursor = next;
    if (ui.input !== 'touch') grid.focusIndex(next);
  }

  // 電腦隊友：隨機挑沒人用的角色
  function teammates(n, taken) {
    const pool = CHARACTERS.filter((c) => !c.hidden && !taken.includes(c.id)).map((c) => c.id);
    const out = [];
    for (let i = 0; i < n; i++) out.push(pool.splice(Math.floor(Math.random() * pool.length), 1)[0] ?? 'xiaocui');
    return out;
  }

  function confirm() {
    if (!done) return pick();
    play('select');
    const ids = picks.map((c) => c.id);
    const characters = [...ids, ...teammates(allies, ids)].slice(0, 4);
    onstart({ characters, humans, settings: { difficulty, world: Number(world), lives: Number(lives) } });
  }

  function randomPick() {
    const pool = CHARACTERS.filter((c) => isUnlocked(c) && !picks.includes(c));
    pick(pool[Math.floor(Math.random() * pool.length)]);
  }

  function back() {
    play('select');
    if (picks.length) picks.pop();
    else onback();
  }

  // Esc／手把 B（由 App 透過 Controls 呼叫）
  export function goBack() {
    back();
  }
</script>

<ScreenFrame title={t('集結夥伴')} {subtitle}>
  <div class="select">
    <div class="roster">
      <CharacterGrid bind:this={grid} bind:cursor {tags} columns={3} onpick={(c) => pick(c)} />
    </div>

    <div class="hero">
      {#key current.id}
        {#if open}
          <img src={current.portraits.happy} alt="" draggable="false" />
        {:else}
          <img class="lock" src={asset('ui/icons/lock.webp')} alt="" />
        {/if}
      {/key}
      <div class="name-brush">
        <b>{open ? current.name : '？？？'}</b>
        <small>{open ? current.intro : t('打倒第一大關的魔王就會出現。')}</small>
      </div>
    </div>

    <div class="info">
      <h3 class="ink-tab">{t('能力')}</h3>
      <div class="stats">
        {#each STATS as [label, key] (key)}
          <div class="stat">
            {t(label)}
            <div class="bar">{#each { length: 5 } as _, i (i)}<i class:on={open && i < current.stats[key]}></i>{/each}</div>
            <b>{open ? current.stats[key] : '?'}</b>
          </div>
        {/each}
      </div>
      <h3 class="ink-tab">{t('闖關設定')}</h3>
      <div class="settings">
        {#if start === 'party'}
          <span>{t('人數')}</span><Stepper options={humanOptions} bind:value={humans} />
        {/if}
        {#if maxAllies > 0}
          <span>{t('電腦隊友')}</span><Stepper options={allyOptions} bind:value={allies} />
        {/if}
        <span>{t('難度')}</span><Stepper options={difficultyOptions} bind:value={difficulty} />
        <span>{t('出發關卡')}</span><Stepper options={worldOptions} bind:value={world} />
        <span>{t('命數')}</span><Stepper options={livesOptions} bind:value={lives} />
      </div>
      {#if !done}<button type="button" class="vbtn small random" onclick={randomPick}>{t('隨機')}</button>{/if}
    </div>

    <div class="slots">
      {#each { length: humans + allies } as _, i (i)}
        {@const c = i < humans ? picks[i] : null}
        {@const now = i === step && !done}
        <div class="slot" style="--c: {SLOT_COLORS[i]}" class:now>
          <span class="no">{i + 1}P</span>
          <div class="pic">
            {#if c}<img src={c.portraits.normal} alt="" />{:else if now && open}<img class="ghost" src={current.portraits.normal} alt="" />{:else}<span>？</span>{/if}
          </div>
          <b>{c ? c.name : i < humans ? (now && open ? current.name : '—') : 'CPU'}</b>
          <em class:ok={c} class:cpu={i >= humans}>{c ? t('已確認') : i >= humans ? t('隨機') : now ? t('選擇中') : t('等待中')}</em>
        </div>
      {/each}
    </div>
  </div>

  {#snippet foot()}
    <BackButton onclick={back} />
    <ControlHint extra={ui.input === 'touch' ? t('點一下看資料，再點一下決定') : ''} />
    <button type="button" class="vbtn primary" bind:this={startButton} disabled={!done && !open} onclick={confirm}>
      {done ? t('出發！') : t('決定')}
    </button>
  {/snippet}
</ScreenFrame>

<style>
  .select {
    flex: 1;
    display: grid;
    grid-template-columns: 30cqw 27cqw 1fr;
    grid-template-rows: minmax(0, 1fr) auto;
    gap: 1.6cqh 1.8cqw;
    min-height: 0;
  }
  .roster {
    display: flex;
    min-height: 0;
  }
  /* 名冊：4 列平均分高度，格子跟著縮，不會被底下的席位蓋住 */
  .roster :global(.grid) {
    gap: 1.2cqh 1cqw;
    grid-template-rows: repeat(4, minmax(0, 1fr));
    align-content: stretch;
    min-height: 0;
  }
  .roster :global(.card) {
    aspect-ratio: auto;
    min-height: 0;
  }

  /* 中間的大頭像：網點底的漫畫格，左下壓一個毛筆名字 */
  .hero {
    position: relative;
    border: 0.42cqh solid var(--navy);
    background:
      radial-gradient(rgba(40, 127, 120, 0.35) 22%, transparent 26%) 0 0 / 1.2cqh 1.2cqh,
      linear-gradient(160deg, #d7e8e2, #f5efdf 70%);
    overflow: hidden;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    min-height: 0;
  }
  .hero img {
    position: absolute;
    left: 50%;
    bottom: 0;
    height: 100%;
    translate: -50% 0;
    animation: slide 0.18s ease-out;
  }
  .hero img.lock {
    height: 30%;
    bottom: 35%;
    opacity: 0.6;
  }
  .name-brush {
    position: relative;
    align-self: flex-end;
    margin: 0 auto 1.6cqh 1.6cqh;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
  }
  .name-brush b {
    background: var(--navy);
    color: var(--on-dark);
    font-size: 6cqh;
    font-style: italic;
    font-weight: 900;
    padding: 0 3cqh 0 1.6cqh;
    line-height: 1.2;
    clip-path: polygon(0 10%, 6% 0, 94% 8%, 100% 30%, 96% 100%, 4% 92%);
  }
  .name-brush small {
    margin-top: 0.4cqh;
    background: var(--cream);
    border: 0.25cqh solid var(--navy);
    padding: 0.2cqh 1.2cqh;
    font-size: 2cqh;
    font-weight: 800;
    transform: rotate(-2deg);
  }

  .info {
    display: flex;
    flex-direction: column;
    gap: 1.2cqh;
    min-height: 0;
    min-width: 0;
  }
  .ink-tab {
    align-self: flex-start;
    margin: 0;
    padding: 0.4cqh 5cqh 0.4cqh 1.8cqh;
    background: var(--navy);
    color: var(--on-dark);
    font-size: 3.2cqh;
    font-weight: 900;
    letter-spacing: 0.3em;
    clip-path: polygon(0 8%, 5% 0, 88% 6%, 100% 0, 95% 50%, 100% 100%, 3% 94%);
  }
  .stats {
    display: flex;
    flex-direction: column;
  }
  .stats .stat {
    padding: 0.8cqh 0;
    border-bottom: 0.2cqh solid var(--navy);
  }
  .settings {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.8cqh 1.6cqw;
    align-items: center;
    font-size: 2.3cqh;
    font-weight: 900;
  }
  .random {
    align-self: flex-end;
  }

  .slots {
    grid-column: 1 / -1;
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1.6cqw;
  }
  .slot {
    position: relative;
    display: grid;
    grid-template-columns: 9cqh 1fr auto;
    align-items: center;
    gap: 1cqw;
    padding: 0.8cqh 1.2cqw 0.8cqh 0.8cqh;
    border: 0.3cqh solid var(--c);
    background: var(--paper-white);
    min-width: 0;
  }
  .slot.now {
    box-shadow: 0 0 0 0.3cqh var(--c);
    animation: pulse 1s ease-in-out infinite alternate;
  }
  .no {
    position: absolute;
    left: -0.3cqh;
    top: -1.8cqh;
    background: var(--c);
    color: #fff;
    font-weight: 900;
    font-size: 2.2cqh;
    padding: 0 1.2cqh 0 0.8cqh;
    clip-path: polygon(0 0, 100% 0, 86% 100%, 0 100%);
    z-index: 1;
  }
  .pic {
    width: 9cqh;
    height: 7.4cqh;
    overflow: hidden;
    background: #efe6cf;
    display: grid;
    place-items: center;
    font-size: 3.4cqh;
    font-weight: 900;
    color: var(--muted);
  }
  .pic img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    object-position: center 15%;
  }
  .pic img.ghost {
    opacity: 0.5;
  }
  .slot b {
    font-size: 2.8cqh;
    font-weight: 900;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .slot em {
    font-style: normal;
    font-weight: 900;
    font-size: 2cqh;
    padding: 0.4cqh 1.2cqh;
    border: 0.25cqh solid var(--navy);
    white-space: nowrap;
  }
  .slot em.ok {
    background: var(--accent);
    color: var(--on-dark);
    border-color: var(--accent);
  }
  .slot em.cpu {
    background: var(--navy);
    color: var(--on-dark);
  }
  @keyframes pulse {
    to {
      box-shadow: 0 0 0 0.6cqh transparent;
    }
  }
  @keyframes slide {
    from {
      opacity: 0.4;
      transform: translateX(1.4cqh);
    }
  }
</style>
