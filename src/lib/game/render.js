// 畫面：背景、角色、敵人、箱子、道具、特效、HUD。只讀取遊戲狀態，不改它。
//
// 狀態格式見 crates/quest-core/src/game.rs（Match 序列化的 JSON）。畫面位置 = (x − 鏡頭, z − y)。
// 敵人與魔王還沒有圖：先用程式畫 Q 版人形（顏色照 stages.js 每一大關換），美術交件後換成動作圖。

import { W, H, Z_TOP, Z_BOTTOM, SLOT_COLORS } from './constants.js';
import { pickLine } from './characters.js';
import { WORLDS, stageLabel } from './stages.js';
import { drawScenery, drawAmbient } from './scenery.js';
import SHEETS from './sheets.json' with { type: 'json' };
import { asset, currentConfig } from '../art-style.svelte.js';
import { loadImage } from '../preload.svelte.js';
import { t, displayName } from '../i18n.svelte.js';

// 內部解析度：一般風格 2 倍（1920×1080）比較銳利；像素風格用較低的倍數再放大（見 style-assets.js）
let SCALE = 2;

const ART_PATHS = {
  spark: 'effects/hit-spark.webp',
  dust: 'effects/sand-puff.webp',
  burst: 'effects/score-burst.webp',
};
const art = (key) => loadImage(asset(ART_PATHS[key]));
let cfg = currentConfig();

// HUD 的字、框、對話泡泡的顏色：跟介面用同一組 CSS 變數，限色風格也一致
// 墨黑 HUD 底、暖白字、青綠血條，低血量朱紅（art-work/ui-concept-v1/DESIGN.md）
const pal = { text: '#f5efdf', ink: '#202329', flash: '#e9b85e', paper: '#fffbf1', cream: '#f5efdf', accent: '#d94b3d', teal: '#287f78', hp: '#2fbf9a', hpLow: '#d94b3d' };
let palStyle = null;
function updatePalette() {
  const style = document.documentElement.dataset.style ?? 'anime90';
  if (style === palStyle) return;
  palStyle = style;
  const css = getComputedStyle(document.documentElement);
  const v = (name, fallback) => css.getPropertyValue(name).trim() || fallback;
  pal.text = v('--on-dark', '#fff');
  pal.ink = v('--navy', '#202329');
  pal.flash = v('--gold', '#e9b85e');
  pal.paper = v('--paper-white', '#fff');
  pal.accent = v('--accent', '#d94b3d');
  pal.hpLow = pal.accent;
  pal.cream = v('--cream', '#f5efdf');
  pal.teal = v('--teal', '#287f78');
}

const ready = (img) => img && img.complete && img.naturalWidth > 0;
const FONT = '"Noto Sans TC", "Microsoft JhengHei", sans-serif';

// ---------- 特效 ----------

const particles = [];
const effects = [];
const bubbles = [];
const floaters = []; // 「+100」之類往上飄的字
let shake = 0;
let flashScreen = 0;

function effect(key, x, y, w, life, { rot = 0, grow = 0.3, alpha = 1, flip = false } = {}) {
  effects.push({ key, x, y, w, life, max: life, rot, grow, alpha, flip });
}

function burst(x, y, color, n, speed, life = 30, gravity = 0.25) {
  for (let i = 0; i < n; i++) {
    const a = Math.random() * Math.PI * 2;
    const s = speed * (0.4 + Math.random() * 0.6);
    particles.push({ x, y, vx: Math.cos(a) * s, vy: Math.sin(a) * s - 1, life, max: life, color, gravity, size: 2 + Math.random() * 3 });
  }
}

function say(slot, text, frames = 110) {
  if (!text) return;
  bubbles.splice(0, bubbles.length, ...bubbles.filter((b) => b.slot !== slot));
  bubbles.push({ slot, text, timer: frames, max: frames });
}

export function resetEffects() {
  particles.length = 0;
  effects.length = 0;
  bubbles.length = 0;
  floaters.length = 0;
  shake = 0;
  flashScreen = 0;
}

const playerOf = (match, slot) => match.players.find((p) => p.slot === slot);
const randomPlayer = (match) => {
  const alive = match.players.filter((p) => !p.out);
  return alive[Math.floor(Math.random() * alive.length)];
};

// 依遊戲事件產生特效與台詞（座標是關卡座標，畫的時候再減鏡頭）
export function handleEvents(match) {
  for (const e of match.events) {
    switch (e.type) {
      case 'hit':
        effect('spark', e.x, e.y, e.big ? 120 : 70, e.big ? 16 : 10, { rot: Math.random() * Math.PI });
        burst(e.x, e.y, '#fff3a0', e.big ? 10 : 5, e.big ? 7 : 5, 16, 0);
        if (e.big) shake = Math.max(shake, 6);
        break;
      case 'hurt':
        effect('spark', e.x, e.y, e.big ? 110 : 64, 10, { rot: Math.random() * Math.PI, alpha: 0.8 });
        burst(e.x, e.y, '#ff8a8a', 6, 5, 16, 0);
        if (e.big) shake = Math.max(shake, 8);
        break;
      case 'step':
        effect('dust', e.x, e.z - 8, 70, 16, { alpha: 0.5, flip: Math.random() < 0.5 });
        break;
      case 'land':
        effect('dust', e.x, e.z - 10, 110, 22, { alpha: 0.6, flip: Math.random() < 0.5 });
        break;
      case 'quake':
        effect('dust', e.x, e.z - 14, 320, 34, { grow: 0.5 });
        shake = Math.max(shake, 16);
        break;
      case 'ko':
        effect('burst', e.x, e.z - 60, e.boss ? 320 : 150, e.boss ? 60 : 30, { grow: 0.5 });
        if (e.boss) {
          shake = Math.max(shake, 20);
          flashScreen = 18;
        }
        if (e.by != null && Math.random() < (e.boss ? 1 : 0.25)) {
          const p = playerOf(match, e.by);
          if (p) say(p.slot, pickLine(p.character, e.boss ? 'win' : 'ko'));
        }
        break;
      case 'break':
        burst(e.x, e.z - 30, '#c08a4a', 14, 6, 30);
        break;
      case 'pickup': {
        const p = playerOf(match, e.slot);
        if (p) floaters.push({ x: p.x, y: p.z - 150, text: e.kind === 'gem' ? '+2000' : t('回復！'), life: 50, max: 50, color: e.kind === 'gem' ? '#7ff0ff' : '#7dff8a' });
        break;
      }
      case 'special': {
        const p = playerOf(match, e.slot);
        if (p) {
          effect('burst', p.x, p.z - 60, 220, 22, { grow: 0.4, alpha: 0.75 });
          // 每個人的必殺顏色、招式名稱
          burst(p.x, p.z - 70, p.character.specialColor, 14, 7, 24, 0);
          floaters.push({ x: p.x, y: p.z - 170, text: p.character.specialName, life: 60, max: 60, color: p.character.specialColor });
        }
        break;
      }
      case 'nosp': {
        // 氣不夠還按必殺：提示一下（同一個人不要一直跳字）
        const p = playerOf(match, e.slot);
        if (p && !floaters.some((f) => f.noSp === p.slot)) {
          floaters.push({ x: p.x, y: p.z - 150, text: t('必殺氣不夠！'), life: 40, max: 40, color: '#c9ccd4', noSp: p.slot });
        }
        break;
      }
      case 'stage': {
        const p = randomPlayer(match);
        if (p) say(p.slot, pickLine(p.character, 'go'), 140);
        break;
      }
      case 'boss': {
        const p = randomPlayer(match);
        if (p) say(p.slot, pickLine(p.character, 'boss'));
        break;
      }
      case 'down': {
        const p = playerOf(match, e.slot);
        if (p) say(p.slot, pickLine(p.character, 'hurt'));
        break;
      }
      case 'clear': {
        const p = randomPlayer(match);
        if (p) say(p.slot, pickLine(p.character, 'win'), 160);
        break;
      }
    }
  }
}

// ---------- 小工具 ----------

function rr(g, x, y, w, h, r) {
  g.beginPath();
  g.roundRect(x, y, w, h, r);
}

function outlinedText(g, text, x, y, size, fill = pal.text, align = 'center') {
  g.font = `900 ${size}px ${FONT}`;
  g.textAlign = align;
  g.textBaseline = 'middle';
  g.lineJoin = 'round';
  g.lineWidth = size / 5;
  g.strokeStyle = pal.ink;
  g.strokeText(text, x, y);
  g.fillStyle = fill;
  g.fillText(text, x, y);
}

function shadow(g, x, z, y, w) {
  g.fillStyle = `rgba(20,20,40,${Math.max(0.08, 0.28 - y / 900)})`;
  g.beginPath();
  g.ellipse(x, z + 2, w * Math.max(0.45, 1 - y / 500), 8, 0, 0, Math.PI * 2);
  g.fill();
}

// 無敵中閃爍
const blinking = (b, frame) => b.inv > 0 && Math.floor(frame / 3) % 2 === 0;

// ---------- 玩家 ----------

// 動作圖集（scripts/import-quest-art.py 產生，格子清單在 sheets.json）：
// 角色／敵人每格在遊戲裡畫 CELL_H 高、魔王 BOSS_H 高；腳底在格子的 496/512（= 744/768）。
const CELL_H = 200;
const BOSS_H = 300;
const ANCHOR = 496 / 512;
// 人物大約多高（頭上標記、對話泡泡的位置用）
const FIG_H = 160;

/** 從圖集畫一格。frame 沒有就用 fallback（再沒有就第一格）；回傳有沒有畫到 */
function drawFrame(g, sheet, frame, x, y, h, facing, { fallback = 'idle', flash = 0, rotate = 0, alpha = 1 } = {}) {
  const img = loadImage(asset(sheet.src));
  if (!ready(img)) return false;
  let i = sheet.frames.indexOf(frame);
  if (i < 0) i = Math.max(0, sheet.frames.indexOf(fallback));
  const c = sheet.cell;
  // 寬姿勢（高踢）交件時縮小了、或整張圖集偏小（長吻、大鉗子）：以腳底為中心放大回來
  // （倍數是 scripts/import-quest-art.py 依交件的實測高度算的）
  h *= (sheet.fix?.[sheet.frames[i]] ?? 1) * (sheet.scale ?? 1);
  g.save();
  g.translate(x, y);
  if (facing < 0) g.scale(-1, 1);
  if (rotate) g.rotate(rotate);
  if (alpha < 1) g.globalAlpha = alpha;
  if (flash > 0) g.filter = 'brightness(2.2)';
  g.drawImage(img, (i % sheet.cols) * c, Math.floor(i / sheet.cols) * c, c, c, -h / 2, -h * ANCHOR, h, h);
  g.restore();
  return true;
}

// 清版動作專用圖集（16 格）的動作
// 走路：有四格（walk1 跨步、walk3 收腳、walk2 跨另一腳、walk4 收腳）就照四格輪播，只有兩格就兩格換快一點
const WALK4 = ['walk1', 'walk3', 'walk2', 'walk4'];
function walkPose(t, sheet) {
  if (sheet?.frames.includes('walk3') && sheet.frames.includes('walk4')) return WALK4[Math.floor(t / 7) % 4];
  return Math.floor(t / 7) % 2 ? 'walk1' : 'walk2';
}
// 走路的上下起伏：跨步時身體低一點、換腳中間高一點（只有兩格圖時用，起伏讓步伐看起來比較連續）
function walkBob(t) {
  return -Math.abs(Math.sin((t / 14) * Math.PI)) * 3;
}

function poseOf(p, sheet) {
  switch (p.act) {
    case 'walk':
      return walkPose(p.t, sheet);
    case 'run':
      // 還沒有跑步圖：用走路的兩格、換得快一點
      if (sheet?.frames.includes('run1')) return Math.floor(p.t / 6) % 2 ? 'run1' : 'run2';
      return Math.floor(p.t / 5) % 2 ? 'walk1' : 'walk2';
    case 'attack':
      if (p.mv === 'dashAttack') return sheet?.frames.includes('dash') ? 'dash' : 'kick';
      return { jab: 'punch1', jab2: 'punch2', kick: 'kick', special: 'special', throw: 'throw' }[p.mv] ?? 'idle';
    case 'grab':
      return p.mv === 'knee' ? 'knee' : 'grab';
    case 'jump':
      return p.mv === 'jumpKick' ? 'jumpkick' : 'jump';
    case 'enter':
      return 'jump';
    case 'hurt':
    case 'rise':
      return 'hurt';
    case 'fall':
      return 'fall';
    case 'down':
    case 'dead':
      return 'down';
    case 'win':
      return 'win';
    default:
      return 'idle';
  }
}

// 還沒有清版圖集的角色：借用排球王的 8 格（idle run1 run2 jump spike dive win lose）
const LEGACY_FRAMES = ['idle', 'run1', 'run2', 'jump', 'spike', 'dive', 'win', 'lose'];
const LEGACY_H = 190;
function legacyPose(p) {
  switch (p.act) {
    case 'walk':
      return Math.floor(p.t / 9) % 2 ? 'run1' : 'run2';
    case 'run':
      return Math.floor(p.t / 5) % 2 ? 'run1' : 'run2';
    case 'attack':
      return p.mv === 'none' ? 'idle' : 'spike';
    case 'grab':
      return p.mv === 'knee' ? 'spike' : 'idle';
    case 'jump':
      return p.mv === 'jumpKick' ? 'spike' : 'jump';
    case 'enter':
      return 'jump';
    case 'hurt':
    case 'rise':
      return 'lose';
    case 'fall':
    case 'down':
    case 'dead':
      return 'dive';
    case 'win':
      return 'win';
    default:
      return 'idle';
  }
}

function drawSpecialAura(g, p, x) {
  if (!(p.act === 'attack' && p.mv === 'special')) return;
  g.save();
  g.globalAlpha = 0.35 + 0.2 * Math.sin(p.t * 0.8);
  g.fillStyle = p.character.specialColor;
  g.beginPath();
  g.ellipse(x, p.z, 70, 16, 0, 0, Math.PI * 2);
  g.fill();
  g.restore();
}

function drawPlayer(g, match, p, cam) {
  if (p.out) return;
  const x = p.x - cam;
  shadow(g, x, p.z, p.y, 40);
  drawSpecialAura(g, p, x);
  if (p.act === 'dead' && Math.floor(p.t / 4) % 2 === 0) return;
  if (blinking(p, match.frame)) return;
  const sheet = SHEETS.characters?.[p.character.id];
  if (sheet) {
    // 出拳時往前一點點、被打時往後退一點
    let lunge = 0;
    if (p.act === 'attack' && p.mv !== 'none') lunge = Math.min(p.t, 4) * 1.2;
    if (p.act === 'hurt') lunge = -Math.max(0, 6 - p.t) * 1.5;
    const pose = poseOf(p, sheet);
    // 借用走路圖跑步時身體往前傾一點，看起來比較像在跑
    const lean = p.act === 'run' && !sheet.frames.includes('run1') ? 0.12 : 0;
    // 四格走路圖本身就有上下起伏（收腳時身體最高），只有兩格時才用程式補
    const bob = p.act === 'walk' && !sheet.frames.includes('walk3') ? walkBob(p.t) : 0;
    drawFrame(g, sheet, pose, x + lunge * p.facing, p.z - p.y + bob, CELL_H, p.facing, { flash: p.flash, rotate: lean });
    return;
  }
  const img = p.character.sprite ? loadImage(p.character.spriteSrc) : null;
  // 圖還沒載入就先不畫（進遊戲前會先載好，見 preload.svelte.js）
  if (!ready(img)) return;
  const i = LEGACY_FRAMES.indexOf(legacyPose(p));
  const fw = img.naturalWidth / LEGACY_FRAMES.length;
  const fh = img.naturalHeight;
  const drawH = LEGACY_H;
  const drawW = drawH * (fw / fh);
  g.save();
  let lunge = 0;
  if (p.act === 'attack' && p.mv !== 'none') lunge = Math.min(p.t, 6) * 2;
  if (p.act === 'hurt') lunge = -Math.max(0, 6 - p.t) * 2;
  g.translate(x + lunge * p.facing, p.z - p.y);
  if (p.facing < 0) g.scale(-1, 1);
  if (p.act === 'attack' && p.mv === 'special') g.scale(Math.cos(p.t * 0.6) >= 0 ? 1 : -1, 1);
  if (p.act === 'fall') g.rotate(-0.3);
  if (p.act === 'down' || p.act === 'dead') g.translate(0, 18);
  if (p.act === 'rise') g.scale(1.05, 0.9);
  if (p.mv === 'jumpKick' && p.act === 'jump') g.rotate(0.25);
  if (p.flash > 0) g.filter = 'brightness(2.2)';
  g.drawImage(img, i * fw, 0, fw, fh, -drawW / 2, -drawH * cfg.spriteAnchorY, drawW, drawH);
  g.restore();
}

// 頭上的 1P／2P 標記
function drawSlotMarker(g, p, cam, you, cpu) {
  if (p.out || p.act === 'dead') return;
  const x = p.x - cam;
  const y = p.z - p.y - FIG_H - 6;
  const color = SLOT_COLORS[p.slot];
  g.fillStyle = color;
  g.beginPath();
  g.moveTo(x - 9, y - 8);
  g.lineTo(x + 9, y - 8);
  g.lineTo(x, y + 4);
  g.fill();
  outlinedText(g, `${p.slot + 1}P`, x, y - 18, you ? 18 : 15, color);
}

// ---------- 敵人（程式畫的 Q 版人形） ----------

function enemyLook(match, e) {
  const w = WORLDS[match.world];
  // 沒有圖時程式畫的 Q 版人形：比例放大 1.25 倍，跟交件的動作圖差不多高
  if (e.kind === 'boss') return { cloth: w.boss.color, pants: w.boss.accent, head: w.boss.color, accent: w.boss.accent, scale: 1.75 * 1.25 };
  const c = w.enemy;
  const scale = (e.kind === 'brute' ? 1.3 : e.kind === 'knife' ? 0.95 : 1) * (e.elite ? 1.25 : 1) * 1.25;
  return { ...c, scale };
}

// 敵人的動作 → 圖集格子
function enemyPose(e, sheet) {
  switch (e.act) {
    case 'walk':
      return walkPose(e.t, sheet);
    case 'attack': {
      // 魔王跳起來落地後的硬直（沒有招式）：用蹲低的 slam 那格當落地的衝擊
      if (e.mv === 'none') return e.kind === 'boss' ? 'slam' : 'idle';
      // 起手格數跟 crates/quest-core 的 move_data 一致：起手時擺出拉手的姿勢，看得出來要打了
      const windup = e.t < ({ slam: 15, bossSlam: 15, shoot: 12, punch: 12, slash: 10, uppercut: 5 }[e.mv] ?? 6);
      if (windup) return 'windup';
      return e.kind === 'boss' ? 'slam' : 'attack';
    }
    case 'charge':
      return 'charge';
    case 'leap':
      return 'leap';
    case 'cast':
      return 'cast';
    case 'vanish':
      return 'vanish';
    case 'hurt':
    case 'held':
    case 'rise':
      return 'hurt';
    case 'fall':
    case 'thrown':
      return 'fall';
    case 'down':
    case 'dead':
      return 'down';
    default:
      return 'idle';
  }
}

// 用交件的圖畫敵人；這一大關這種敵人還沒圖就回傳 false（改用程式畫的）
function drawEnemySprite(g, match, e, x) {
  const world = WORLDS[match.world].id;
  const boss = e.kind === 'boss';
  const sheet = boss ? SHEETS.bosses?.[world] : SHEETS.enemies?.[`${world}/${e.kind}`];
  if (!sheet) return false;
  const pose = enemyPose(e, sheet);
  const h = (boss ? BOSS_H : CELL_H) * (e.elite ? 1.25 : 1);
  const fallback = pose === 'charge' || pose === 'leap' ? 'walk1' : pose === 'cast' ? 'windup' : 'idle';
  const vanish = pose === 'vanish' && !sheet.frames.includes('vanish');
  if (vanish && e.t > 6 && e.t < 34) return true;
  const lift = e.act === 'thrown' ? e.t * 0.35 : 0;
  // 瞬移（40 格，第 20 格換位置）：前半淡出、後半在新位置淡入
  const alpha = e.act === 'vanish' ? Math.max(0.08, e.t < 20 ? 1 - e.t / 20 : (e.t - 20) / 20) : 1;
  if (e.elite) {
    // 小頭目：外圈杏黃光
    g.save();
    g.filter = 'drop-shadow(0 0 3px #e9b85e) drop-shadow(0 0 3px #e9b85e)';
    drawFrame(g, sheet, pose, x, e.z - e.y, h, e.facing, { fallback, flash: e.flash, rotate: lift, alpha });
    g.restore();
  } else {
    drawFrame(g, sheet, pose, x, e.z - e.y, h, e.facing, { fallback, flash: e.flash, rotate: lift, alpha });
  }
  if (e.act === 'attack' && pose === 'windup' && (boss || e.kind === 'brute' || e.elite)) {
    outlinedText(g, '!', x, e.z - e.y - h * 0.82, 26, pal.flash);
  }
  return true;
}

function drawEnemy(g, match, e, cam) {
  const x = e.x - cam;
  const look = enemyLook(match, e);
  const sheetScale = e.kind === 'boss' ? 1.6 : e.elite ? 1.25 : 1;
  if (SHEETS.enemies?.[`${WORLDS[match.world].id}/${e.kind}`] || (e.kind === 'boss' && SHEETS.bosses?.[WORLDS[match.world].id])) {
    shadow(g, x, e.z, e.y, 34 * sheetScale);
    if (e.act === 'dead' && Math.floor(e.t / 4) % 2 === 0) return;
    if (drawEnemySprite(g, match, e, x)) return;
  }
  const s = look.scale;
  shadow(g, x, e.z, e.y, 30 * s);
  if (e.act === 'dead' && Math.floor(e.t / 4) % 2 === 0) return;
  if (e.act === 'vanish' && e.t > 6 && e.t < 34) {
    // 瞬移中：只剩紫色的煙
    g.fillStyle = 'rgba(160,60,255,0.35)';
    g.beginPath();
    g.ellipse(x, e.z - 60, 40, 70, 0, 0, Math.PI * 2);
    g.fill();
    return;
  }
  g.save();
  g.translate(x, e.z - e.y);
  if (e.facing < 0) g.scale(-1, 1);
  g.scale(s, s);
  const lying = e.act === 'down' || e.act === 'dead';
  if (lying) {
    g.rotate(-Math.PI / 2);
    g.translate(-18, -20);
  } else if (e.act === 'fall' || e.act === 'thrown') {
    g.rotate(e.act === 'thrown' ? e.t * 0.35 : -0.6);
  } else if (e.act === 'hurt' || e.act === 'held') {
    g.rotate(-0.15);
  } else if (e.act === 'charge') {
    g.rotate(0.25);
  }
  if (e.flash > 0) g.filter = 'brightness(2.4)';

  const walk = e.act === 'walk' || e.act === 'charge' ? Math.sin(e.t * 0.35) : 0;
  // 招式前搖：往後縮；出手：手伸長
  const attacking = e.act === 'attack' && e.mv !== 'none';
  const windup = attacking && e.t < (e.mv === 'slam' || e.mv === 'bossSlam' ? 15 : 6);
  const reach = attacking && !windup ? 26 : windup ? -6 : 0;

  // 腿
  g.fillStyle = look.pants;
  g.fillRect(-12 + walk * 6, -34, 10, 34);
  g.fillRect(3 - walk * 6, -34, 10, 34);
  // 身體
  g.fillStyle = look.cloth;
  rr(g, -18, -78, 36, 48, 10);
  g.fill();
  g.strokeStyle = 'rgba(20,20,40,0.55)';
  g.lineWidth = 2.5;
  g.stroke();
  // 手
  g.fillStyle = look.head;
  g.beginPath();
  g.arc(16 + reach, -56 + (windup ? -6 : 0), 7, 0, Math.PI * 2);
  g.fill();
  g.stroke();
  // 武器／特徵
  if (e.kind === 'knife') {
    g.fillStyle = '#e8eef8';
    g.fillRect(20 + reach, -60, 24, 5);
  }
  if (e.kind === 'thrower') {
    g.fillStyle = look.accent;
    g.beginPath();
    g.arc(-16, -64, 10, 0, Math.PI * 2);
    g.fill();
  }
  // 頭
  g.fillStyle = look.head;
  g.beginPath();
  g.arc(2, -96, 22, 0, Math.PI * 2);
  g.fill();
  g.stroke();
  // 眼睛：凶狠的眉毛
  g.fillStyle = '#1a1a2a';
  g.fillRect(8, -100, 5, 7);
  g.fillRect(-4, -100, 5, 7);
  g.strokeStyle = '#1a1a2a';
  g.lineWidth = 2.5;
  g.beginPath();
  g.moveTo(-6, -108);
  g.lineTo(2, -104);
  g.moveTo(16, -108);
  g.lineTo(8, -104);
  g.stroke();
  if (e.act === 'hurt' || e.act === 'fall' || lying) {
    // 被打：眼睛變成 ×
    g.fillStyle = look.head;
    g.fillRect(-6, -102, 22, 10);
    g.strokeStyle = '#1a1a2a';
    for (const ex of [-2, 10]) {
      g.beginPath();
      g.moveTo(ex - 3, -100);
      g.lineTo(ex + 3, -94);
      g.moveTo(ex + 3, -100);
      g.lineTo(ex - 3, -94);
      g.stroke();
    }
  }
  // 頭上的配件
  g.fillStyle = look.accent;
  if (e.kind === 'boss') drawBossCrown(g, match.world);
  else if (e.kind === 'brute') g.fillRect(-18, -118, 40, 7);
  else g.fillRect(-14, -120, 32, 8);
  if (e.elite) {
    g.strokeStyle = pal.flash;
    g.lineWidth = 3;
    g.beginPath();
    g.arc(2, -96, 27, 0, Math.PI * 2);
    g.stroke();
  }
  g.restore();
  // 招式快出手時的「！」提示
  if (windup && (e.kind === 'boss' || e.kind === 'brute' || e.elite)) outlinedText(g, '!', x, e.z - e.y - 130 * s, 26, pal.flash);
}

function drawBossCrown(g, world) {
  switch (WORLDS[world].id) {
    case 'city':
      g.fillRect(-20, -122, 44, 9); // 墨鏡帶
      break;
    case 'forest':
      for (const dx of [-14, 18]) {
        g.beginPath();
        g.arc(dx, -116, 9, 0, Math.PI * 2);
        g.fill();
      }
      break;
    case 'sea':
      g.beginPath();
      g.moveTo(-6, -116);
      g.lineTo(8, -146);
      g.lineTo(18, -114);
      g.fill();
      break;
    case 'ice':
    case 'castle':
      for (const dx of [-14, 2, 18]) {
        g.beginPath();
        g.moveTo(dx - 6, -114);
        g.lineTo(dx, -138);
        g.lineTo(dx + 6, -114);
        g.fill();
      }
      break;
    case 'mine':
      g.fillRect(-20, -124, 44, 12);
      g.fillStyle = '#ff9a2e';
      g.fillRect(-4, -134, 10, 10);
      break;
  }
}

// ---------- 箱子、道具、飛行道具 ----------

const PROP_LOOK = {
  city: ['#9aa3b5', '#6c768a'], // 垃圾桶
  forest: ['#8a5a32', '#5e3a1e'], // 樹樁
  sea: ['#e0a4c6', '#a8638a'], // 大貝殼
  ice: ['#e4f4ff', '#9cc6e6'], // 雪塊
  mine: ['#6a6a76', '#3a3a44'], // 礦車
  castle: ['#8a6a4a', '#5a3a2a'], // 木桶
};

// 有交件的圖就畫圖（sheets.json 的 props／items／shots），沒有就用程式畫的
function drawImageProp(g, src, x, y, w, h, { flip = false, shake = 0 } = {}) {
  const img = loadImage(asset(src));
  if (!ready(img)) return false;
  g.save();
  g.translate(x + shake, y);
  if (flip) g.scale(-1, 1);
  g.drawImage(img, -w / 2, -h, w, h);
  g.restore();
  return true;
}

function drawProp(g, match, pr, cam) {
  const x = pr.x - cam;
  const world = WORLDS[match.world].id;
  const [a, b] = PROP_LOOK[world];
  shadow(g, x, pr.z, 0, 30);
  if (SHEETS.props?.includes(`${world}-crate`)) {
    // 256×256 的圖，底部中心在 (128, 244)：遊戲裡畫 84 高
    const shake = pr.flash > 0 ? (Math.random() - 0.5) * 4 : 0;
    const img = loadImage(asset(`props/${world}-crate.webp`));
    if (ready(img)) {
      g.save();
      g.translate(x + shake, pr.z);
      if (pr.hp < 2) g.filter = 'brightness(0.85)';
      g.drawImage(img, -42, -84 * (244 / 256), 84, 84);
      g.restore();
      return;
    }
  }
  g.save();
  g.translate(x + (pr.flash > 0 ? (Math.random() - 0.5) * 4 : 0), pr.z);
  g.fillStyle = a;
  rr(g, -26, -58, 52, 58, 6);
  g.fill();
  g.strokeStyle = b;
  g.lineWidth = 4;
  g.stroke();
  g.beginPath();
  g.moveTo(-26, -30);
  g.lineTo(26, -30);
  g.stroke();
  if (pr.hp < 2) {
    g.strokeStyle = 'rgba(0,0,0,0.5)';
    g.lineWidth = 2;
    g.beginPath();
    g.moveTo(-10, -58);
    g.lineTo(2, -36);
    g.lineTo(-6, -18);
    g.stroke();
  }
  g.restore();
}

function drawItem(g, it, cam, frame) {
  const x = it.x - cam;
  const bob = Math.sin(frame * 0.12 + it.id) * 3;
  shadow(g, x, it.z, it.y, 16);
  if (SHEETS.items?.includes(it.kind) && drawImageProp(g, `props/items/${it.kind}.webp`, x, it.z - it.y - 2 + bob, 44, 44)) return;
  g.save();
  g.translate(x, it.z - it.y - 16 + bob);
  if (it.kind === 'apple') {
    g.fillStyle = '#ff3b4a';
    g.beginPath();
    g.arc(0, 0, 12, 0, Math.PI * 2);
    g.fill();
    g.fillStyle = '#3fbf4a';
    g.fillRect(1, -18, 8, 6);
  } else if (it.kind === 'meat') {
    g.fillStyle = '#b5652a';
    g.beginPath();
    g.ellipse(-4, 0, 16, 11, -0.3, 0, Math.PI * 2);
    g.fill();
    g.fillStyle = '#fff3e0';
    g.fillRect(10, -4, 12, 6);
  } else if (it.kind === 'feast') {
    g.fillStyle = '#ffffff';
    g.beginPath();
    g.ellipse(0, 6, 26, 8, 0, 0, Math.PI * 2);
    g.fill();
    g.fillStyle = '#d0803a';
    g.beginPath();
    g.arc(0, -4, 16, Math.PI, 0);
    g.fill();
  } else {
    g.fillStyle = '#5fe3ff';
    g.beginPath();
    g.moveTo(0, -14);
    g.lineTo(12, 0);
    g.lineTo(0, 14);
    g.lineTo(-12, 0);
    g.fill();
    g.fillStyle = 'rgba(255,255,255,0.7)';
    g.fillRect(-3, -8, 4, 6);
  }
  g.restore();
}

const SHOT_COLORS = { rock: '#8a8478', bubble: 'rgba(180,240,255,0.7)', ice: '#bfe9ff', fire: '#ff8a2e', dark: '#b03aff' };
// 雜兵丟的東西每一大關不一樣（白色都市丟瓶子…）；魔王的泡泡、冰柱、火球、暗黑球
const THROWN = ['bottle', 'stone', 'ink', 'snowball', 'bomb', 'magic'];
function drawShot(g, s, cam, frame, world) {
  const x = s.x - cam;
  const y = s.z - s.y;
  shadow(g, x, s.z, s.y, 12);
  const name = s.kind === 'rock' ? THROWN[world] : s.kind;
  if (SHEETS.shots?.includes(name)) {
    const img = loadImage(asset(`props/shots/${name}.webp`));
    if (ready(img)) {
      g.save();
      g.translate(x, y);
      if (s.vx < 0) g.scale(-1, 1);
      // 丟出去的東西會轉，魔王的彈不轉
      if (s.kind === 'rock') g.rotate(frame * 0.3);
      g.drawImage(img, -24, -24, 48, 48);
      g.restore();
      return;
    }
  }
  g.save();
  g.translate(x, y);
  g.fillStyle = SHOT_COLORS[s.kind];
  if (s.kind === 'moon') {
    // 月華的月光波：發光的新月
    if (s.vx < 0) g.scale(-1, 1);
    g.shadowColor = '#c9d4ff';
    g.shadowBlur = 24;
    g.fillStyle = '#eef2ff';
    g.beginPath();
    g.arc(0, 0, 40, -Math.PI / 2, Math.PI / 2);
    g.arc(-14, 0, 36, Math.PI / 2, -Math.PI / 2, true);
    g.fill();
    g.restore();
    return;
  }
  if (s.kind === 'ice') {
    g.rotate(s.vx > 0 ? 0 : Math.PI);
    g.beginPath();
    g.moveTo(18, 0);
    g.lineTo(-12, -7);
    g.lineTo(-12, 7);
    g.fill();
  } else {
    const r = s.kind === 'rock' ? 9 : 14 + Math.sin(frame * 0.4) * 2;
    if (s.kind === 'fire' || s.kind === 'dark') {
      g.globalAlpha = 0.4;
      g.beginPath();
      g.arc(0, 0, r * 1.6, 0, Math.PI * 2);
      g.fill();
      g.globalAlpha = 1;
    }
    g.beginPath();
    g.arc(0, 0, r, 0, Math.PI * 2);
    g.fill();
    if (s.kind === 'bubble') {
      g.strokeStyle = '#ffffff';
      g.lineWidth = 2;
      g.stroke();
    }
  }
  g.restore();
}

// ---------- HUD ----------

function hpBar(g, x, y, w, h, hp, max, frame = 0) {
  g.fillStyle = '#3a3e46';
  g.fillRect(x, y, w, h);
  const r = Math.max(0, hp / max);
  const low = r < 0.3;
  // 低血量：朱紅＋短暫閃一下（不是只靠閃爍提示）
  g.fillStyle = low ? (Math.floor(frame / 12) % 4 === 0 ? '#ff8b7e' : pal.hpLow) : pal.hp;
  g.fillRect(x, y, w * r, h);
  g.fillStyle = 'rgba(255,255,255,0.28)';
  g.fillRect(x, y, w * r, h / 3);
  g.strokeStyle = 'rgba(245,239,223,0.5)';
  g.lineWidth = 1;
  g.strokeRect(x + 0.5, y + 0.5, w - 1, h - 1);
}

// 必殺氣：杏黃，集滿了換成角色的必殺顏色並閃一下（滿了才能放必殺）
const SP_MAX = 100;
function spBar(g, x, y, w, h, sp, color, frame) {
  g.fillStyle = '#3a3e46';
  g.fillRect(x, y, w, h);
  const full = sp >= SP_MAX;
  g.fillStyle = full ? (Math.floor(frame / 8) % 2 ? '#ffffff' : color) : pal.flash;
  g.fillRect(x, y, w * Math.min(1, sp / SP_MAX), h);
  if (full) {
    g.strokeStyle = color;
    g.lineWidth = 1.5;
    g.strokeRect(x - 0.5, y - 0.5, w + 1, h + 1);
  }
}

// 毛筆刷過的長方形（標籤、章）：邊緣微微不齊
function brush(g, x, y, w, h, color) {
  g.fillStyle = color;
  g.beginPath();
  g.moveTo(x + 4, y + h * 0.1);
  g.lineTo(x + w * 0.3, y);
  g.lineTo(x + w * 0.7, y + 2);
  g.lineTo(x + w - 2, y - 1);
  g.lineTo(x + w + 6, y + h * 0.35);
  g.lineTo(x + w - 2, y + h * 0.6);
  g.lineTo(x + w + 3, y + h);
  g.lineTo(x + w * 0.5, y + h - 2);
  g.lineTo(x + 2, y + h + 1);
  g.lineTo(x - 4, y + h * 0.55);
  g.closePath();
  g.fill();
}

// 頂端的隊員牌：肖像、1P～4P／CPU、名稱、HP、剩餘生命、七位數分數
const CARD = { w: 228, h: 58, gap: 8, top: 8 };
function drawPlayerPanel(g, p, names, cpu, frame) {
  const x = 10 + p.slot * (CARD.w + CARD.gap);
  const y = CARD.top;
  const color = SLOT_COLORS[p.slot];
  g.fillStyle = 'rgba(32,35,41,0.92)';
  g.fillRect(x, y, CARD.w, CARD.h);
  g.strokeStyle = 'rgba(245,239,223,0.55)';
  g.lineWidth = 1.5;
  g.strokeRect(x + 1.5, y + 1.5, CARD.w - 3, CARD.h - 3);
  // 座位色條：肖像左邊一條
  g.fillStyle = color;
  g.fillRect(x, y, 4, CARD.h);
  const face = loadImage(p.character.portraits[p.act === 'hurt' || p.act === 'fall' || p.act === 'down' ? 'surprised' : p.act === 'win' ? 'happy' : 'normal']);
  if (ready(face)) {
    g.save();
    g.beginPath();
    g.rect(x + 7, y + 5, 48, 48);
    g.clip();
    g.fillStyle = color;
    g.fillRect(x + 7, y + 5, 48, 48);
    if (p.out) g.filter = 'grayscale(1) brightness(0.6)';
    g.drawImage(face, x + 7, y + 3, 50, 50);
    g.restore();
    g.strokeStyle = color;
    g.lineWidth = 2;
    g.strokeRect(x + 7, y + 5, 48, 48);
  }
  // 1P 標籤（斜切）
  const tx = x + 62;
  g.fillStyle = color;
  g.beginPath();
  g.moveTo(tx, y + 6);
  g.lineTo(tx + 30, y + 6);
  g.lineTo(tx + 25, y + 22);
  g.lineTo(tx, y + 22);
  g.closePath();
  g.fill();
  g.font = `900 13px ${FONT}`;
  g.textAlign = 'left';
  g.textBaseline = 'middle';
  g.fillStyle = '#fff';
  g.fillText(`${p.slot + 1}P`, tx + 4, y + 14.5);
  const name = cpu ? `CPU ${p.character.name}` : names?.[p.slot] ? displayName(names[p.slot]) : p.character.name;
  g.font = `800 14px ${FONT}`;
  g.fillStyle = pal.text;
  let shown = name;
  while (g.measureText(shown).width > CARD.w - 104 && shown.length > 1) shown = shown.slice(0, -1);
  g.fillText(shown === name ? name : `${shown}…`, tx + 36, y + 14.5);
  if (p.out) {
    outlinedText(g, 'GAME OVER', x + 140, y + 40, 15, '#ff8b7e');
    return;
  }
  hpBar(g, tx, y + 25, CARD.w - 70, 9, p.hp, p.maxHp, frame);
  spBar(g, tx, y + 36, CARD.w - 70, 5, p.sp ?? 0, p.character.specialColor, frame);
  g.font = `800 13px ${FONT}`;
  g.fillStyle = pal.text;
  g.fillText(`♥ ×${Math.max(0, p.lives)}`, tx, y + 49);
  g.textAlign = 'right';
  g.font = `900 14px ${FONT}`;
  g.fillText(String(p.score).padStart(7, '0'), x + CARD.w - 8, y + 49);
}

// 隊員牌下面的關卡標籤：「白色都市　1-1 白色大道」
function drawStageTag(g, match) {
  const w = WORLDS[match.world];
  const text = `${t(w.name)}   ${stageLabel(match.world, match.sub)} ${t(w.subs[match.sub])}`;
  g.font = `900 14px ${FONT}`;
  const tw = g.measureText(text).width;
  brush(g, 8, CARD.top + CARD.h + 8, tw + 34, 24, 'rgba(32,35,41,0.85)');
  g.textAlign = 'left';
  g.textBaseline = 'middle';
  g.fillStyle = pal.text;
  g.fillText(text, 20, CARD.top + CARD.h + 20.5);
}

// 魔王戰：底部一條窄的名字＋HP，讓出地面
function drawBossBar(g, match) {
  const boss = match.enemies.find((e) => e.id === match.boss);
  if (!boss) return;
  const w = 440;
  const x = (W - w) / 2;
  const y = H - 30;
  g.fillStyle = 'rgba(32,35,41,0.88)';
  g.fillRect(x - 12, y - 22, w + 24, 38);
  g.fillStyle = pal.accent;
  g.fillRect(x - 12, y - 22, 4, 38);
  g.font = `900 14px ${FONT}`;
  g.textAlign = 'left';
  g.textBaseline = 'middle';
  g.fillStyle = pal.text;
  g.fillText(`BOSS  ${t(WORLDS[match.world].boss.name)}`, x, y - 10);
  g.fillStyle = '#3a3e46';
  g.fillRect(x, y + 1, w, 9);
  g.fillStyle = pal.accent;
  g.fillRect(x, y + 1, w * Math.max(0, boss.hp / boss.maxHp), 9);
}

// 關卡開頭的章節牌、過關的「突破！」章、全軍覆沒
function drawChapter(g, match, alpha) {
  const w = WORLDS[match.world];
  g.save();
  g.globalAlpha = alpha;
  const slide = (1 - Math.min(1, alpha)) * -60;
  g.translate(slide, 0);
  brush(g, W / 2 - 250, H / 2 - 74, 500, 74, 'rgba(32,35,41,0.92)');
  outlinedText(g, t(w.name), W / 2, H / 2 - 37, 40, pal.cream);
  g.fillStyle = pal.accent;
  g.fillRect(W / 2 - 170, H / 2 + 8, 340, 34);
  g.font = `900 20px ${FONT}`;
  g.textAlign = 'center';
  g.textBaseline = 'middle';
  g.fillStyle = '#fff';
  g.fillText(`STAGE ${stageLabel(match.world, match.sub)}  ${t(w.subs[match.sub])}`, W / 2, H / 2 + 25.5);
  g.restore();
}

function drawStamp(g, text, sub, alpha, scale = 1) {
  g.save();
  g.globalAlpha = alpha;
  g.translate(W / 2, H / 2 - 10);
  g.rotate(-0.08);
  g.scale(scale, scale);
  g.strokeStyle = pal.accent;
  g.lineWidth = 6;
  g.fillStyle = 'rgba(245,239,223,0.92)';
  g.fillRect(-190, -62, 380, 124);
  g.strokeRect(-182, -54, 364, 108);
  g.font = `900 64px ${FONT}`;
  g.textAlign = 'center';
  g.textBaseline = 'middle';
  g.fillStyle = pal.accent;
  g.fillText(text, 0, sub ? -10 : 0);
  if (sub) {
    g.font = `800 17px ${FONT}`;
    g.fillStyle = pal.ink;
    g.fillText(sub, 0, 36);
  }
  g.restore();
}

function drawBubbles(g, match, cam) {
  for (const b of bubbles) {
    const p = playerOf(match, b.slot);
    if (!p || p.out) continue;
    const px = p.x - cam;
    const alpha = Math.min(1, b.timer / 15, (b.max - b.timer) / 6);
    g.save();
    g.globalAlpha = alpha;
    g.font = `700 18px ${FONT}`;
    const tw = g.measureText(b.text).width;
    const w = tw + 26;
    const h = 36;
    const x = Math.max(10, Math.min(W - w - 10, px - w / 2));
    const y = Math.max(100, p.z - p.y - FIG_H - 70);
    g.fillStyle = pal.paper;
    g.strokeStyle = pal.ink;
    g.lineWidth = 3;
    rr(g, x, y, w, h, 12);
    g.fill();
    g.stroke();
    g.beginPath();
    g.moveTo(px - 7, y + h - 1.5);
    g.lineTo(px, y + h + 12);
    g.lineTo(px + 7, y + h - 1.5);
    g.fill();
    g.stroke();
    g.fillStyle = pal.ink;
    g.textAlign = 'center';
    g.textBaseline = 'middle';
    g.fillText(b.text, x + w / 2, y + h / 2 + 1);
    g.restore();
  }
}

function drawHud(g, match, opts) {
  match.players.forEach((p) => drawPlayerPanel(g, p, opts.names, opts.cpu?.[p.slot], match.frame));
  if (match.phase === 'play') drawStageTag(g, match);
  drawBossBar(g, match);
  const w = WORLDS[match.world];
  // 清場：右側出現小型「GO →」章
  if (match.go && Math.floor(match.frame / 20) % 3 !== 0) {
    g.save();
    g.translate(W - 110, H / 2 - 40);
    g.rotate(-0.06);
    g.fillStyle = pal.accent;
    g.fillRect(-58, -26, 116, 52);
    g.strokeStyle = pal.cream;
    g.lineWidth = 3;
    g.strokeRect(-52, -20, 104, 40);
    g.font = `900 30px ${FONT}`;
    g.textAlign = 'center';
    g.textBaseline = 'middle';
    g.fillStyle = '#fff';
    g.fillText('GO →', 0, 1);
    g.restore();
  }
  if (match.phase === 'intro') {
    const k = 140 - match.timer;
    drawChapter(g, match, Math.min(1, k / 12, match.timer / 18));
  } else if (match.phase === 'clear') {
    const k = 200 - match.timer;
    // 蓋章：一開始大大的，啪一下縮回原尺寸
    drawStamp(g, t('突破！'), `${stageLabel(match.world, match.sub)} ${t(w.subs[match.sub])}`, Math.min(1, k / 6), Math.max(1, 1.6 - k * 0.08));
  } else if (match.phase === 'over') {
    g.save();
    g.globalAlpha = Math.min(1, match.timer / 30);
    brush(g, W / 2 - 230, H / 2 - 44, 460, 88, 'rgba(32,35,41,0.92)');
    outlinedText(g, 'GAME OVER', W / 2, H / 2, 52, pal.accent);
    g.restore();
  } else if (match.phase === 'ending') {
    drawStamp(g, t('破關！'), t('六大關全部突破！'), Math.min(1, match.timer / 20));
  }
}

// ---------- 主畫面 ----------

// opts：{ you: 自己的座位（連線時）, names: 各座位的玩家名稱（連線時）, cpu: 各座位是不是電腦 }
export function draw(canvas, match, opts = {}) {
  cfg = currentConfig();
  updatePalette();
  if (SCALE !== cfg.canvasScale || canvas.width !== Math.round(W * cfg.canvasScale)) {
    SCALE = cfg.canvasScale;
    resize(canvas);
  }
  const g = canvas.getContext('2d');
  g.imageSmoothingEnabled = !cfg.pixelated;
  g.setTransform(SCALE, 0, 0, SCALE, 0, 0);
  if (!match) {
    g.fillStyle = '#0c1d4b';
    g.fillRect(0, 0, W, H);
    return;
  }
  if (shake > 0) {
    g.translate((Math.random() - 0.5) * shake, (Math.random() - 0.5) * shake);
    shake *= 0.85;
    if (shake < 0.5) shake = 0;
  }
  const cam = match.camera;
  drawScenery(g, match.world, match.sub, cam, SCALE);

  // 依前後（z）排序：靠後的先畫
  const things = [];
  for (const pr of match.props) things.push([pr.z, () => drawProp(g, match, pr, cam)]);
  for (const it of match.items) things.push([it.z - 1, () => drawItem(g, it, cam, match.frame)]);
  for (const e of match.enemies) things.push([e.z + (e.act === 'held' ? 0.5 : 0), () => drawEnemy(g, match, e, cam)]);
  for (const p of match.players) things.push([p.z, () => drawPlayer(g, match, p, cam)]);
  for (const s of match.shots) things.push([s.z + 0.2, () => drawShot(g, s, cam, match.frame, match.world)]);
  things.sort((a, b) => a[0] - b[0]);
  for (const [, fn] of things) fn();

  for (const fx of effects) {
    const img = art(fx.key);
    if (!ready(img)) continue;
    const k = 1 - fx.life / fx.max;
    const w = fx.w * (1 + fx.grow * k);
    const h = w * (img.naturalHeight / img.naturalWidth);
    g.save();
    g.globalAlpha = fx.alpha * Math.min(1, (fx.life / fx.max) * 1.6);
    g.translate(fx.x - cam, fx.y);
    g.rotate(fx.rot);
    if (fx.flip) g.scale(-1, 1);
    g.drawImage(img, -w / 2, -h / 2, w, h);
    g.restore();
  }
  for (const pt of particles) {
    g.globalAlpha = pt.life / pt.max;
    g.fillStyle = pt.color;
    g.fillRect(pt.x - cam - pt.size / 2, pt.y - pt.size / 2, pt.size, pt.size);
  }
  g.globalAlpha = 1;
  for (const f of floaters) {
    g.globalAlpha = Math.min(1, f.life / 15);
    // 字不要超出畫面左右
    g.font = `900 20px ${FONT}`;
    const half = g.measureText(f.text).width / 2 + 8;
    outlinedText(g, f.text, Math.min(W - half, Math.max(half, f.x - cam)), f.y - (f.max - f.life) * 0.8, 20, f.color);
  }
  g.globalAlpha = 1;
  drawAmbient(g, match.world, match.sub, match.frame);
  if (flashScreen > 0) {
    g.fillStyle = `rgba(255,255,255,${flashScreen / 24})`;
    g.fillRect(0, 0, W, H);
  }

  for (const p of match.players) drawSlotMarker(g, p, cam, opts.you === p.slot, opts.cpu?.[p.slot]);
  drawBubbles(g, match, cam);
  drawHud(g, match, opts);
}

// 跟模擬同步，每個影格呼叫一次
export function tick() {
  for (let i = particles.length - 1; i >= 0; i--) {
    const pt = particles[i];
    pt.x += pt.vx;
    pt.y += pt.vy;
    pt.vy += pt.gravity;
    if (--pt.life <= 0) particles.splice(i, 1);
  }
  for (let i = effects.length - 1; i >= 0; i--) if (--effects[i].life <= 0) effects.splice(i, 1);
  for (let i = bubbles.length - 1; i >= 0; i--) if (--bubbles[i].timer <= 0) bubbles.splice(i, 1);
  for (let i = floaters.length - 1; i >= 0; i--) if (--floaters[i].life <= 0) floaters.splice(i, 1);
  if (flashScreen > 0) flashScreen--;
}

export function resize(canvas) {
  canvas.width = Math.round(W * SCALE);
  canvas.height = Math.round(H * SCALE);
  canvas.style.imageRendering = cfg.pixelated ? 'pixelated' : 'auto';
}

export { Z_TOP, Z_BOTTOM };
