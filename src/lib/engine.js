// 遊戲主迴圈：固定 60 fps 讀輸入、推進遊戲，每個畫面影格繪圖。
// 遊戲本身在 Rust（crates/quest-core）：
//   單機 → 瀏覽器裡的 WebAssembly（crates/quest-wasm）
//   連線 → 房主每個影格送來狀態，這裡只送輸入、負責畫
// 兩邊的狀態格式一樣，所以畫面只有一套。跟 Svelte 無關，UI 透過回傳的方法控制它。

import init, { LocalMatch } from './local/pkg/quest_wasm.js';
import * as input from './game/input.js';
import * as render from './game/render.js';
import { play } from './game/audio.js';
import { getCharacter } from './game/characters.js';
import { net, onFrame, sendInput } from './net.svelte.js';

const FRAME = 1000 / 60;
// 全軍覆沒／破關後先停一下才跳選單
const END_DELAY = { over: 90, ending: 150 };

// 遊戲事件 → 音效（src/lib/audio/sound.svelte.js 的 sfx）
const EVENT_SOUND = {
  swing: 'swing',
  jump: 'jump',
  land: 'land',
  hurt: 'hurt',
  special: 'special',
  grab: 'cursor',
  throw: 'heavy',
  ko: 'ko',
  break: 'crack',
  pickup: 'pickup',
  shoot: 'shoot',
  quake: 'quake',
  go: 'go',
  boss: 'alarm',
  nosp: 'deny',
  weaponbreak: 'crack',
};

const wasmReady = init();

// 引擎送來的 JSON → 畫面用的物件（角色 id 換成角色資料）
function hydrate(state) {
  state.players.forEach((p) => (p.character = getCharacter(p.character)));
  return state;
}

/**
 * onStage(match)：新的一小關開始（換 BGM）
 * onEnd(kind, match)：kind = over（全軍覆沒，可以接關）| ending（破關）
 * onClear(match)：小關突破
 */
export function createEngine(canvas, { onEnd, onStage = () => {}, onClear = () => {}, onBoss = () => {} }) {
  render.resize(canvas);

  // idle：在選單（選單蓋住整個畫面，不跑遊戲也不出聲）
  let state = 'idle'; // idle | local | paused | ended | online
  let local = null; // { game: LocalMatch, humans: [讀輸入的函式 或 null] }
  let match = null;
  let endTimer = 0;
  let pressCount = input.createPressCounter();
  let ready = false;
  wasmReady.then(() => (ready = true));

  // 必殺氣上一格是多少（集滿的那一刻提示一聲，只提示真人玩家，電腦隊友不吵）
  let lastSp = {};
  const isHuman = (i, p) => (state === 'online' ? p.slot === net.seat : local && !local.cpu[i]);

  function advance(m) {
    m.players.forEach((p, i) => {
      if (p.sp >= 100 && (lastSp[p.slot] ?? 100) < 100 && !p.out && isHuman(i, p)) play('gauge');
      lastSp[p.slot] = p.sp;
    });
    for (const e of m.events) {
      if (e.type === 'hit') play(e.big ? 'heavy' : 'punch');
      else if (EVENT_SOUND[e.type]) play(EVENT_SOUND[e.type]);
      if (e.type === 'stage') {
        endTimer = 0;
        onStage(m);
      } else if (e.type === 'clear') onClear(m);
      else if (e.type === 'boss') onBoss(m);
    }
    render.handleEvents(m);
    render.tick();
  }

  // 結束畫面：over／ending 停一下才通知
  function checkEnd(m) {
    const delay = END_DELAY[m.phase];
    if (!delay) {
      endTimer = 0;
      return false;
    }
    if (++endTimer === delay) {
      onEnd(m.phase, m);
      return true;
    }
    return false;
  }

  // 收到房主的影格
  const offFrame = onFrame((s) => {
    if (state !== 'online') return;
    match = hydrate(s);
    advance(match);
    checkEnd(match);
  });

  function simulate() {
    if (!ready) return;
    if (state === 'local' || state === 'ended') {
      const bits = local.humans.map((h) => (h && state === 'local' ? input.toBits(h()) : 0));
      local.game.step(new Uint32Array(bits));
      match = hydrate(JSON.parse(local.game.state()));
      advance(match);
      if (state === 'local' && checkEnd(match)) state = 'ended';
    } else if (state === 'online') {
      // 只送自己的輸入，遊戲在房主那邊跑
      if (net.seat != null) {
        // 自己一個人操作：所有鍵盤配置、所有手把、觸控按鈕都算
        const i = input.read(0, { keymaps: input.SOLO_KEYMAPS, pads: 'all', touch: 0 });
        sendInput(input.toBits({ ...i, attackPressed: false, jumpPressed: false, specialPressed: false }), pressCount(i));
      }
    }
  }

  let last = performance.now();
  let acc = 0;
  let raf = 0;
  function loop(now) {
    acc += Math.min(100, now - last);
    last = now;
    let steps = 0;
    while (acc >= FRAME) {
      simulate();
      acc -= FRAME;
      steps++;
    }
    // 沒跑模擬的影格（高更新率螢幕）不要清掉「剛按下」，否則會吃掉按鍵
    if (steps > 0) input.endFrame();
    if (state !== 'idle') draw();
    raf = requestAnimationFrame(loop);
  }
  function draw() {
    const online = state === 'online';
    render.draw(canvas, match, {
      you: online ? net.seat : null,
      names: online ? net.room?.seats.map((s) => s?.name) : null,
      cpu: online ? net.room?.seats.map((s) => s?.kind === 'bot') : local?.cpu,
    });
  }
  raf = requestAnimationFrame(loop);

  return {
    /**
     * 單機：characters = 角色 id（依 1P、2P… 排）；humans = 前幾位是真人（其餘是電腦隊友）
     * settings = { difficulty, world, lives }
     */
    async startLocal({ characters, humans, settings }) {
      await wasmReady;
      local?.game.free();
      const game = new LocalMatch(JSON.stringify(characters), JSON.stringify(settings));
      const readers = characters.map((_, i) => {
        if (i >= humans) {
          game.setAi(i, settings.difficulty);
          return null;
        }
        // 一個人玩：所有鍵盤配置、所有手把、觸控都算 1P
        if (humans === 1) return () => input.read(0, { keymaps: input.SOLO_KEYMAPS, pads: 'all', touch: 0 });
        // 多人：1P、2P 各一組鍵盤；第幾支手把就是第幾個玩家；觸控只有 1P、2P
        return () => input.read(i, { keymaps: i < 2 ? [i] : [], pads: [i], touch: i < 2 ? i : null });
      });
      local = { game, humans: readers, cpu: readers.map((r) => !r) };
      match = null;
      endTimer = 0;
      render.resetEffects();
      state = 'local';
    },
    // 暫停選單：從這一小關的開頭重來（單機）
    restartStageLocal() {
      if (!local) return false;
      local.game.restartStage();
      endTimer = 0;
      render.resetEffects();
      state = 'local';
      return true;
    },
    // 全軍覆沒後接關（單機）
    continueLocal() {
      if (!local || !local.game.continueGame()) return false;
      endTimer = 0;
      render.resetEffects();
      state = 'local';
      return true;
    },
    // 連線：畫房主送來的遊戲
    startOnline() {
      if (state === 'online') return;
      match = null;
      endTimer = 0;
      pressCount = input.createPressCounter();
      render.resetEffects();
      state = 'online';
    },
    pause() {
      if (state === 'local') state = 'paused';
    },
    resume() {
      if (state === 'paused') state = 'local';
    },
    // 回到選單：停掉遊戲（不再出聲）
    stop() {
      local?.game.free();
      local = null;
      match = null;
      render.resetEffects();
      state = 'idle';
    },
    destroy() {
      cancelAnimationFrame(raf);
      offFrame();
    },
    get match() {
      return match;
    },
  };
}
