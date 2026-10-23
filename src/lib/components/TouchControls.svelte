<script>
  // 觸控按鈕：左邊搖桿（八方向移動），右邊「攻擊」「跳」「必殺」三顆鈕。
  // 雙人同一台平板時，兩組各放在自己那一側。
  import { asset } from '../art-style.svelte.js';
  import { t } from '../i18n.svelte.js';
  import { onDestroy } from 'svelte';
  import { setTouch, pressTouch, releaseTouch, clearTouch } from '../game/input.js';

  // slots：要顯示哪幾組（[0] 單人；[0, 1] 雙人同一台）
  let { slots = [0], onpause = () => {} } = $props();

  // 抓住這根手指，滑出按鈕外也繼續算（瀏覽器不給抓就算了，不影響操作）
  function capture(e) {
    try {
      e.currentTarget.setPointerCapture(e.pointerId);
    } catch {
      // 沒有對應的指標
    }
  }
  const split = $derived(slots.length > 1);

  // 搖桿可推動的半徑：底盤半徑的 0.7 倍（按下時依實際大小算）
  const stickR = [60, 60];
  let knobs = $state([{ x: 0, y: 0 }, { x: 0, y: 0 }]);
  const stickPointer = [null, null];
  const stickOrigin = [null, null];

  function stickDown(slot, e) {
    e.preventDefault();
    capture(e);
    const r = e.currentTarget.getBoundingClientRect();
    stickPointer[slot] = e.pointerId;
    stickOrigin[slot] = { x: r.left + r.width / 2, y: r.top + r.height / 2 };
    stickR[slot] = (r.width / 2) * 0.7;
    stickMove(slot, e);
  }

  function stickMove(slot, e) {
    if (stickPointer[slot] !== e.pointerId) return;
    let dx = e.clientX - stickOrigin[slot].x;
    let dy = e.clientY - stickOrigin[slot].y;
    const d = Math.hypot(dx, dy);
    const R = stickR[slot];
    if (d > R) {
      dx = (dx / d) * R;
      dy = (dy / d) * R;
    }
    knobs[slot] = { x: dx, y: dy };
    const nx = dx / R;
    const ny = dy / R;
    setTouch(slot, { left: nx < -0.35, right: nx > 0.35, up: ny < -0.4, down: ny > 0.4 });
  }

  function stickUp(slot, e) {
    if (stickPointer[slot] !== e.pointerId) return;
    stickPointer[slot] = null;
    knobs[slot] = { x: 0, y: 0 };
    setTouch(slot, { left: false, right: false, up: false, down: false });
  }

  let pressed = $state({});
  function button(slot, kind, down, e) {
    e.preventDefault();
    if (down) capture(e);
    pressed[`${slot}${kind}`] = down;
    if (down) {
      pressTouch(slot, kind);
      navigator.vibrate?.(8);
    } else releaseTouch(slot, kind);
  }

  const BUTTONS = [
    ['special', '必殺'],
    ['jump', '跳'],
    ['attack', '攻擊'],
  ];

  onDestroy(clearTouch);
</script>

<div class="touch" class:split>
  <button type="button" tabindex="-1" class="pause" aria-label={t('暫停')} onclick={onpause}><img src="{asset('ui/icons/pause.webp')}" alt="" /></button>
  {#each slots as slot (slot)}
    {@const mirror = split && slot === 1}
    <div
      class="stick"
      class:p2={split && slot === 1}
      role="presentation"
      onpointerdown={(e) => stickDown(slot, e)}
      onpointermove={(e) => stickMove(slot, e)}
      onpointerup={(e) => stickUp(slot, e)}
      onpointercancel={(e) => stickUp(slot, e)}
    >
      <div class="knob" style="transform: translate({knobs[slot].x}px, {knobs[slot].y}px)"></div>
      {#if split}<span class="tag">{slot + 1}P</span>{/if}
    </div>
    <div class="buttons" class:mirror class:p1={split && slot === 0}>
      {#each BUTTONS as [kind, label] (kind)}
        <button
          type="button"
          tabindex="-1"
          class={kind}
          class:on={pressed[`${slot}${kind}`]}
          onpointerdown={(e) => button(slot, kind, true, e)}
          onpointerup={(e) => button(slot, kind, false, e)}
          onpointercancel={(e) => button(slot, kind, false, e)}
          oncontextmenu={(e) => e.preventDefault()}
        >{t(label)}</button>
      {/each}
    </div>
  {/each}
</div>

<style>
  .touch {
    position: absolute;
    inset: 0;
    pointer-events: none;
    touch-action: none;
    user-select: none;
    -webkit-user-select: none;
    z-index: 4;
  }
  .pause {
    position: absolute;
    top: 2cqh;
    left: 50%;
    transform: translateX(-50%);
    margin-top: 5cqh;
    width: 8cqh;
    height: 8cqh;
    padding: 0;
    border-radius: 50%;
    opacity: 0.85;
    pointer-events: auto;
    box-shadow: none;
    background: var(--cream);
    border: 0.4cqh solid var(--navy);
  }
  .pause img {
    width: 70%;
    height: 70%;
  }
  .stick,
  .buttons {
    position: absolute;
    bottom: 4cqh;
    pointer-events: auto;
    touch-action: none;
  }
  .stick {
    left: 3cqw;
    width: 26cqh;
    height: 26cqh;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.22);
    border: 0.5cqh solid rgba(43, 34, 51, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .knob {
    width: 11cqh;
    height: 11cqh;
    border-radius: 50%;
    background: rgba(255, 250, 240, 0.85);
    border: 0.5cqh solid var(--ink);
    pointer-events: none;
  }
  .tag {
    position: absolute;
    top: -3.6cqh;
    color: var(--on-dark);
    font-weight: 900;
    text-shadow: 0 0 0.6cqh #000;
  }
  .buttons {
    right: 3cqw;
    display: flex;
    gap: 2.4cqh;
    align-items: flex-end;
  }
  .buttons button {
    width: 15cqh;
    height: 15cqh;
    border-radius: 50%;
    padding: 0;
    font-size: 5cqh;
    font-weight: 900;
    opacity: 0.8;
    touch-action: none;
    box-shadow: 0 0.6cqh 0 var(--ink);
  }
  .buttons .attack {
    width: 18cqh;
    height: 18cqh;
    margin-bottom: 6cqh;
    background: var(--accent);
    color: var(--on-dark);
  }
  .buttons .jump {
    background: var(--accent2);
  }
  .buttons .special {
    width: 12cqh;
    height: 12cqh;
    font-size: 3.6cqh;
    margin-bottom: 14cqh;
    background: var(--gold);
  }
  .buttons button.on {
    transform: translateY(0.4cqh) scale(0.94);
    opacity: 1;
  }

  /* 雙人：1P 的按鈕放在左半邊靠中間，2P 的搖桿放右下角 */
  .split .stick {
    width: 22cqh;
    height: 22cqh;
  }
  .split .buttons button {
    width: 12cqh;
    height: 12cqh;
    font-size: 4cqh;
  }
  .split .buttons .attack {
    width: 14cqh;
    height: 14cqh;
  }
  .split .buttons .special {
    width: 10cqh;
    height: 10cqh;
    font-size: 3cqh;
  }
  .buttons.p1 {
    right: auto;
    left: 22cqw;
  }
  .stick.p2 {
    left: auto;
    right: 3cqw;
  }
  .buttons.mirror {
    right: 22cqw;
    flex-direction: row-reverse;
  }
</style>
