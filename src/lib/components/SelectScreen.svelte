<script>
  // 單機選角：依序選每位真人玩家的角色（電腦隊友隨機），右邊看角色資料、調闖關設定
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
  import CharacterDetail from './CharacterDetail.svelte';
  import { t } from '../i18n.svelte.js';

  // onstart({ characters, humans, settings })、onback()
  let { mode, onstart, onback } = $props();

  // 真人人數：一人闖關 1、多人同樂 2～4、電腦示範 0
  // 只看進來時的模式（換模式會重新建這個畫面）
  const start = untrack(() => mode);
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
  const maxAllies = $derived(Math.max(mode === 'watch' ? 1 : 0, 4 - humans));
  const allyChoices = $derived(Array.from({ length: maxAllies + 1 }, (_, i) => i).filter((n) => mode !== 'watch' || n > 0));
  $effect(() => {
    if (allies > maxAllies) allies = maxAllies;
    if (picks.length > humans) picks.length = humans;
  });
  // 正在選第幾位真人（全部選完 = humans）
  const step = $derived(Math.min(picks.length, humans));
  const done = $derived(step >= humans);
  const tags = $derived(picks.reduce((all, c, i) => ({ ...all, [c.id]: [...(all[c.id] ?? []), `${i + 1}P`] }), {}));

  const title = $derived(done ? t('闖關設定') : humans > 1 ? t('{n}P 選角', { n: step + 1 }) : t('選擇角色'));
  const subtitle = $derived(done ? t('調好設定就出發吧！') : t('選好角色後按「決定」'));

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

<ScreenFrame {title} {subtitle}>
  <div class="panel grow">
    <div class="panel-head">{t('角色一覽')}</div>
    <div class="panel-body">
      <CharacterGrid bind:this={grid} bind:cursor {tags} onpick={(c) => pick(c)} />
    </div>
    <div class="slots">
      {#each { length: humans + allies } as _, i (i)}
        {@const c = i < humans ? picks[i] : null}
        <div class="slot" style="--c: {SLOT_COLORS[i]}" class:now={i === step && !done}>
          <span class="who">{i < humans ? `${i + 1}P` : 'CPU'}</span>
          {#if c}<img src={c.portraits.normal} alt="" />{:else}<span class="q">？</span>{/if}
        </div>
      {/each}
    </div>
  </div>

  <div class="side">
    <CharacterDetail character={current} />
    <div class="panel">
      <div class="panel-head">{t('闖關設定')}</div>
      <div class="panel-body settings">
        {#if mode === 'party'}
          <label class="field">{t('人數')}
            <select class="vsel" bind:value={humans}>
              {#each [2, 3, 4] as n (n)}<option value={n}>{t('{n} 人', { n })}</option>{/each}
            </select>
          </label>
        {/if}
        {#if maxAllies > 0}
          <label class="field">{t('電腦隊友')}
            <select class="vsel" bind:value={allies}>
              {#each allyChoices as n (n)}<option value={n}>{t('{n} 位', { n })}</option>{/each}
            </select>
          </label>
        {/if}
        <label class="field">{t('難度')}
          <select class="vsel" bind:value={difficulty}>
            <option value="easy">{t('簡單')}</option>
            <option value="normal">{t('普通')}</option>
            <option value="hard">{t('困難')}</option>
          </select>
        </label>
        <label class="field">{t('出發關卡')}
          <select class="vsel" bind:value={world}>
            {#each WORLDS.slice(0, maxWorld + 1) as w, i (w.id)}<option value={i}>{i + 1}. {t(w.name)}</option>{/each}
          </select>
        </label>
        <label class="field">{t('命數')}
          <select class="vsel" bind:value={lives}>
            {#each [1, 3, 5] as n (n)}<option value={n}>{n}</option>{/each}
          </select>
        </label>
        {#if !done}<button type="button" class="vbtn small" onclick={randomPick}>{t('隨機')}</button>{/if}
      </div>
    </div>
  </div>

  {#snippet foot()}
    <BackButton onclick={back} />
    <ControlHint extra={ui.input === 'touch' ? t('點一下看資料，再點一下決定') : ''} />
    <button type="button" class="vbtn primary" bind:this={startButton} disabled={!done && !isUnlocked(current)} onclick={confirm}>
      {done ? t('出發！') : t('決定')}
    </button>
  {/snippet}
</ScreenFrame>

<style>
  .grow {
    flex: 1.5;
  }
  .side {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1.6cqh;
    min-width: 0;
  }
  .settings {
    flex-direction: row;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: 0.6cqh 1.6cqw;
    padding-top: 0.8cqh;
    padding-bottom: 1cqh;
  }
  .side :global(.detail) {
    flex: 1 0 auto;
  }
  .settings .field {
    flex: 1 1 40%;
  }
  .slots {
    display: flex;
    gap: 1.2cqw;
    padding: 1cqh 1.6cqh 1.4cqh;
    justify-content: center;
  }
  .slot {
    position: relative;
    width: 10cqh;
    height: 10cqh;
    border: 0.45cqh solid var(--c);
    background: var(--navy);
    display: grid;
    place-items: center;
    overflow: hidden;
  }
  .slot.now {
    outline: 0.4cqh solid var(--gold);
    animation: blink 0.8s ease-in-out infinite alternate;
  }
  .slot img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .slot .who {
    position: absolute;
    left: 0;
    top: 0;
    background: var(--c);
    color: #fff;
    font-weight: 900;
    font-size: 1.8cqh;
    padding: 0 0.6cqh;
    z-index: 1;
  }
  .slot .q {
    font-size: 4cqh;
    font-weight: 900;
    color: var(--on-dark);
  }
  @keyframes blink {
    to {
      outline-color: transparent;
    }
  }
</style>
