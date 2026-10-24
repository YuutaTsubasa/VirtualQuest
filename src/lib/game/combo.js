// 連段數與小關評價（只在畫面這邊算：用遊戲事件 hit／hurt／stage／clear，所以單機、連線都一樣）。
//
// 連段：同一個人打中之後 COMBO_GAP 格內又打中就接下去；被打就歸零。
// 評價：每一小關結束時依時間、被打幾次、最高連段算 S／A／B／C，最佳評價存在 localStorage。

const COMBO_GAP = 60;
const PAR_SECONDS = 100;
const BEST_KEY = 'vq_best_rank';
const RANKS = ['C', 'B', 'A', 'S'];

let start = 0;
let hurts = 0;
let combos = {};
/** 剛結束的小關評價（蓋章時顯示）：{ rank, seconds, maxCombo, hurts, best, newBest } */
export let result = null;

function bestTable() {
  try {
    return JSON.parse(localStorage.getItem(BEST_KEY) ?? '{}') ?? {};
  } catch {
    return {};
  }
}

function saveBest(table) {
  try {
    localStorage.setItem(BEST_KEY, JSON.stringify(table));
  } catch {
    // 私密視窗等存不了就算了
  }
}

/** 每一格的遊戲事件（render.js 的 handleEvents 呼叫） */
export function onEvents(match) {
  for (const e of match.events) {
    if (e.type === 'stage') {
      start = match.frame;
      hurts = 0;
      combos = {};
      result = null;
    } else if (e.type === 'hit' && e.by != null) {
      const c = (combos[e.by] ??= { n: 0, last: -999, max: 0 });
      if (match.frame - c.last > COMBO_GAP) c.n = 0;
      c.n += 1;
      c.last = match.frame;
      c.max = Math.max(c.max, c.n);
    } else if (e.type === 'hurt') {
      hurts += 1;
      if (combos[e.slot]) combos[e.slot].n = 0;
    } else if (e.type === 'clear') {
      result = rate(match, e.world, e.sub);
    }
  }
}

/** 這個人目前的連段數（3 下以上才顯示；斷掉了回傳 0） */
export function comboOf(slot, frame) {
  const c = combos[slot];
  if (!c || frame - c.last > COMBO_GAP || c.n < 3) return 0;
  return c.n;
}

function rate(match, world, sub) {
  const seconds = Math.round((match.frame - start) / 60);
  const players = Math.max(1, match.players.length);
  const maxCombo = Math.max(0, ...Object.values(combos).map((c) => c.max));
  // 100 分起算：比標準時間慢扣分、每人每被打一次扣分、連段加分（最多 +20）
  const points = 100 - Math.max(0, seconds - PAR_SECONDS) * 0.5 - (hurts / players) * 5 + Math.min(20, maxCombo * 1.2);
  const rank = points >= 95 ? 'S' : points >= 80 ? 'A' : points >= 60 ? 'B' : 'C';
  const key = `${world}-${sub}`;
  const table = bestTable();
  const before = table[key];
  const newBest = !before || RANKS.indexOf(rank) > RANKS.indexOf(before);
  if (newBest) {
    table[key] = rank;
    saveBest(table);
  }
  return { rank, seconds, maxCombo, hurts, best: newBest ? rank : before, newBest: newBest && !!before };
}

export function resetCombo() {
  combos = {};
  result = null;
}
