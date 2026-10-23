// 遊戲中的輸入：鍵盤、手把、觸控按鈕合在一起。每個影格呼叫 read(slot) 取得該玩家的輸入。
// （選單的操作在 src/lib/components/Controls.svelte）

// 1P、2P 各一組；一個人玩時三組都能用（方向鍵＋Z X C 是很多人習慣的配置）
const KEYMAPS = [
  // 1P：WASD 移動，J 攻擊、K 跳、L 必殺
  { left: ['KeyA'], right: ['KeyD'], up: ['KeyW'], down: ['KeyS'], attack: ['KeyJ'], jump: ['KeyK'], special: ['KeyL'] },
  // 2P：方向鍵，數字鍵 1 2 3（或 , . /）
  {
    left: ['ArrowLeft'], right: ['ArrowRight'], up: ['ArrowUp'], down: ['ArrowDown'],
    attack: ['Numpad1', 'Comma'], jump: ['Numpad2', 'Period'], special: ['Numpad3', 'Slash'],
  },
  // 一個人玩時的另一種：方向鍵＋Z 攻擊、X 跳、C 必殺，空白鍵也是跳
  { left: ['ArrowLeft'], right: ['ArrowRight'], up: ['ArrowUp'], down: ['ArrowDown'], attack: ['KeyZ'], jump: ['KeyX', 'Space'], special: ['KeyC'] },
];
export const SOLO_KEYMAPS = [0, 1, 2];

const held = new Set();
const pressedThisFrame = new Set();

const BUTTONS = ['attack', 'jump', 'special'];
const blank = () => ({ left: false, right: false, up: false, down: false, attack: false });
const touch = [blank(), blank()];
const touchPressed = [new Set(), new Set()];

function typing(target) {
  return !!target?.closest?.('input, textarea, select, [contenteditable]');
}

window.addEventListener('keydown', (e) => {
  if (e.repeat || typing(e.target)) return;
  held.add(e.code);
  pressedThisFrame.add(e.code);
  // 遊戲中空白鍵、方向鍵不要捲動頁面
  if (e.code === 'Space' || e.code.startsWith('Arrow')) e.preventDefault?.();
});
window.addEventListener('keyup', (e) => held.delete(e.code));
window.addEventListener('blur', () => {
  held.clear();
  touch.forEach((t) => Object.assign(t, blank()));
});

// ---------- 手把（標準配置：X / RB 攻擊、A 跳、B / Y 必殺、十字鍵或左搖桿移動） ----------

const padPrev = new Map();

function standardPads() {
  let pads = [];
  try {
    pads = navigator.getGamepads?.() ?? [];
  } catch {
    // 某些瀏覽器在非安全環境會擋
  }
  return [...pads].filter((p) => p?.connected && (!p.mapping || p.mapping === 'standard'));
}

function readPad(pad) {
  const ax = pad.axes[0] ?? 0;
  const ay = pad.axes[1] ?? 0;
  const btn = (i) => !!pad.buttons[i]?.pressed;
  const now = { attack: btn(2) || btn(5), jump: btn(0) || btn(4), special: btn(1) || btn(3) || btn(7) };
  const prev = padPrev.get(pad.index) ?? {};
  padPrev.set(pad.index, now);
  return {
    left: ax < -0.4 || btn(14),
    right: ax > 0.4 || btn(15),
    up: ay < -0.5 || btn(12),
    down: ay > 0.5 || btn(13),
    attack: now.attack,
    attackPressed: now.attack && !prev.attack,
    jumpPressed: now.jump && !prev.jump,
    specialPressed: now.special && !prev.special,
  };
}

// ---------- 觸控（TouchControls.svelte 呼叫） ----------

export function setTouch(slot, patch) {
  Object.assign(touch[slot], patch);
}
/** 觸控按鈕按下：button = attack | jump | special */
export function pressTouch(slot, button) {
  touchPressed[slot].add(button);
  if (button === 'attack') touch[slot].attack = true;
}
export function releaseTouch(slot, button) {
  if (button === 'attack') touch[slot].attack = false;
}
export function clearTouch() {
  touch.forEach((t) => Object.assign(t, blank()));
}

/**
 * 讀一個玩家的輸入。
 * keymaps：要讀哪幾組鍵盤；pads：'all'（所有手把）或手把順序的陣列（第幾支手把，不是 index）；
 * touch：讀哪一組觸控按鈕。
 */
export function read(slot, { keymaps = [slot], pads = [slot], touch: touchSlot = slot } = {}) {
  const any = (codes) => codes.some((c) => held.has(c));
  const anyPressed = (codes) => codes.some((c) => pressedThisFrame.has(c));
  const input = { left: false, right: false, up: false, down: false, attack: false, attackPressed: false, jumpPressed: false, specialPressed: false };
  const merge = (src) => {
    for (const key of Object.keys(input)) input[key] ||= !!src[key];
  };
  for (const k of keymaps) {
    const map = KEYMAPS[k];
    if (!map) continue;
    merge({
      left: any(map.left),
      right: any(map.right),
      up: any(map.up),
      down: any(map.down),
      attack: any(map.attack),
      attackPressed: anyPressed(map.attack),
      jumpPressed: anyPressed(map.jump),
      specialPressed: anyPressed(map.special),
    });
  }
  const connected = standardPads();
  const chosen = pads === 'all' ? connected : pads.map((i) => connected[i]).filter(Boolean);
  for (const pad of chosen) merge(readPad(pad));
  if (touchSlot != null && touch[touchSlot]) {
    merge(touch[touchSlot]);
    for (const b of BUTTONS) if (touchPressed[touchSlot].has(b)) input[`${b}Pressed`] = true;
  }
  return input;
}

// 一個模擬影格結束後清掉「剛按下」的紀錄
export function endFrame() {
  pressedThisFrame.clear();
  touchPressed.forEach((s) => s.clear());
}

// 轉成送給引擎的位元：1 左、2 右、4 上、8 下、16 攻擊（按住）、32 攻擊、64 跳、128 必殺（剛按下）
export function toBits(i) {
  return (
    (i.left ? 1 : 0) |
    (i.right ? 2 : 0) |
    (i.up ? 4 : 0) |
    (i.down ? 8 : 0) |
    (i.attack ? 16 : 0) |
    (i.attackPressed ? 32 : 0) |
    (i.jumpPressed ? 64 : 0) |
    (i.specialPressed ? 128 : 0)
  );
}

// 連線時「按下」改送累計次數（各 8 位元：攻擊、跳、必殺），封包掉了也不會漏按
export function createPressCounter() {
  let a = 0;
  let j = 0;
  let s = 0;
  return (i) => {
    if (i.attackPressed) a = (a + 1) & 0xff;
    if (i.jumpPressed) j = (j + 1) & 0xff;
    if (i.specialPressed) s = (s + 1) & 0xff;
    return a | (j << 8) | (s << 16);
  };
}
