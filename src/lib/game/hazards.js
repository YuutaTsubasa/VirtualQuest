// 關卡機關的樣子：有畫好的圖（public/props/hazards/、各畫風的 styles/<畫風>/props/hazards/）就用圖，沒有就用程式畫的。
// 機關的規則在 crates/quest-core（HazardKind：geyser 噴口、spikes 尖刺、log 滾木；Prop.barrel 油桶）。
// 狀態：idle 平時、warn 發光預告、active 作用中。座標是遊戲座標（x 已經減過鏡頭）。
// 圖的規格見 art-inbox/requests/2026-10-29-1200-style-gaps.md。

import { asset, artStyle } from '../art-style.svelte.js';
import { hasAsset } from '../style-assets.js';
import { loadImage } from '../preload.svelte.js';

const INK = '#202329';

/** 這個畫風（或 90 年代版）有這張機關圖、而且載好了，就回傳圖 */
function art(name) {
  const path = `props/hazards/${name}.webp`;
  if (!hasAsset(artStyle.id, path) && !hasAsset('anime90', path)) return null;
  const img = loadImage(asset(path));
  return img && img.complete && img.naturalWidth > 0 ? img : null;
}

/**
 * 把圖畫上去：圖上 (ax, ay) 那一點對齊遊戲座標 (x, z)，scale 是「遊戲單位／圖的像素」。w 是一格在規格裡的寬（像素畫風的圖是邏輯解析度、比較小，用它換算回規格座標）。
 * clipTop：只畫圖的 y ≥ clipTop 那一段（岩漿柱、尖刺從下往上長出來用）
 */
function drawArt(g, img, x, z, { w, ax, ay, scale, frames = 1, frame = 0, clipTop = 0 }) {
  const k = img.naturalWidth / frames / w; // 像素畫風的圖是邏輯解析度（比較小）
  const fw = img.naturalWidth / frames;
  const sy = clipTop * k;
  g.drawImage(img, frame * fw, sy, fw, img.naturalHeight - sy, x - ax * scale, z - (ay - clipTop) * scale, w * scale, (img.naturalHeight / k - clipTop) * scale);
}

/** 白色都市的油桶：紅色鐵桶＋黃黑警示條；快破了會抖 */
export function drawBarrel(g, x, z, flash) {
  const shake = flash > 0 ? (Math.random() - 0.5) * 4 : 0;
  g.save();
  g.translate(x + shake, z);
  g.lineWidth = 2.5;
  g.strokeStyle = INK;
  g.fillStyle = '#c8402f';
  g.beginPath();
  g.roundRect(-22, -64, 44, 62, 6);
  g.fill();
  g.stroke();
  // 警示條
  g.save();
  g.beginPath();
  g.rect(-22, -42, 44, 12);
  g.clip();
  g.fillStyle = '#ffd23f';
  g.fillRect(-22, -42, 44, 12);
  g.fillStyle = INK;
  for (let k = -30; k < 30; k += 10) {
    g.beginPath();
    g.moveTo(k, -30);
    g.lineTo(k + 6, -42);
    g.lineTo(k + 11, -42);
    g.lineTo(k + 5, -30);
    g.fill();
  }
  g.restore();
  g.strokeRect(-22, -42, 44, 12);
  // 上緣
  g.fillStyle = '#e0604a';
  g.beginPath();
  g.ellipse(0, -64, 22, 6, 0, 0, Math.PI * 2);
  g.fill();
  g.stroke();
  g.restore();
}

/** 噴口：平時是地上的裂縫，預告時發光冒煙，作用時噴出岩漿柱 */
function drawGeyser(g, h, x, frame) {
  // 圖：洞口寬 210 px＝68 單位；平時／預告 256×128、洞口中心 (128, 100)；噴發 256×640、洞口中心 (128, 612)
  const scale = 68 / 210;
  if (h.state === 'active') {
    const img = art('geyser-active');
    if (img) {
      const k = h.t < 6 ? h.t / 6 : h.t > 20 ? Math.max(0, (28 - h.t) / 8) : 1;
      // 柱子從洞口往上長：只畫洞口以上 k 比例的那一段（洞口本身一直都在）
      drawArt(g, img, x, h.z, { w: 256, ax: 128, ay: 612, scale, clipTop: 582 - 552 * k });
      return;
    }
  } else {
    const warnOn = h.state === 'warn' && Math.floor(frame / 4) % 2;
    const img = art(warnOn ? 'geyser-warn' : 'geyser-idle');
    if (img) {
      drawArt(g, img, x, h.z, { w: 256, ax: 128, ay: 100, scale });
      return;
    }
  }
  const glow = h.state === 'warn' ? 0.45 + 0.35 * Math.sin(frame * 0.5) : h.state === 'active' ? 1 : 0.15;
  g.save();
  g.translate(x, h.z);
  g.fillStyle = '#2a1712';
  g.beginPath();
  g.ellipse(0, 0, 34, 9, 0, 0, Math.PI * 2);
  g.fill();
  g.fillStyle = `rgba(255, 120, 30, ${glow})`;
  g.beginPath();
  g.ellipse(0, 0, 26, 6, 0, 0, Math.PI * 2);
  g.fill();
  if (h.state === 'active') {
    // 岩漿柱：一開始衝高，最後縮回去
    const k = h.t < 6 ? h.t / 6 : h.t > 20 ? Math.max(0, (28 - h.t) / 8) : 1;
    const top = -170 * k;
    const grad = g.createLinearGradient(0, top, 0, 0);
    grad.addColorStop(0, 'rgba(255, 230, 120, 0.95)');
    grad.addColorStop(1, 'rgba(255, 80, 20, 0.95)');
    g.fillStyle = grad;
    g.shadowColor = 'rgba(255, 140, 40, 0.9)';
    g.shadowBlur = 24;
    g.beginPath();
    g.moveTo(-22, 0);
    g.quadraticCurveTo(-12 + Math.sin(frame) * 3, top * 0.55, -18, top + 10);
    // 頂端噴開：幾顆岩漿團
    g.quadraticCurveTo(-6, top - 14, 0, top);
    g.quadraticCurveTo(6, top - 14, 18, top + 10);
    g.quadraticCurveTo(12 + Math.cos(frame) * 3, top * 0.55, 22, 0);
    g.closePath();
    g.fill();
    g.shadowBlur = 0;
    g.fillStyle = 'rgba(255, 200, 80, 0.9)';
    for (let k = 0; k < 3; k++) {
      const a = frame * 0.3 + k * 2.1;
      g.beginPath();
      g.arc(Math.cos(a) * 20, top - 6 + Math.sin(a * 1.3) * 10, 5, 0, Math.PI * 2);
      g.fill();
    }
  }
  g.restore();
}

/** 尖刺：平時是有洞的鐵板，預告時閃紅，作用時刺出來 */
function drawSpikes(g, h, x, frame) {
  // 圖：256×256，鐵板中心 (128, 220)、寬 230 px＝76 單位；尖刺最高到 y≈80
  const up = h.state === 'active' ? (h.t < 4 ? h.t / 4 : h.t > 22 ? Math.max(0, (28 - h.t) / 6) : 1) : 0;
  const scale = 76 / 230;
  const plate = art(h.state === 'warn' && Math.floor(frame / 5) % 2 ? 'spikes-warn' : 'spikes-idle');
  const spikes = up > 0 ? art('spikes-active') : null;
  if (plate && (up === 0 || spikes)) {
    if (up > 0) drawArt(g, spikes, x, h.z, { w: 256, ax: 128, ay: 220, scale, clipTop: 200 - 120 * up });
    else drawArt(g, plate, x, h.z, { w: 256, ax: 128, ay: 220, scale });
    return;
  }
  g.save();
  g.translate(x, h.z);
  g.fillStyle = h.state === 'warn' && Math.floor(frame / 5) % 2 ? '#7a2a3a' : '#4a4458';
  g.strokeStyle = INK;
  g.lineWidth = 2;
  g.beginPath();
  g.ellipse(0, 0, 38, 11, 0, 0, Math.PI * 2);
  g.fill();
  g.stroke();
  for (let k = -2; k <= 2; k++) {
    const sx = k * 13;
    const sz = (k % 2) * 4;
    if (up > 0) {
      g.fillStyle = '#d8dce6';
      g.beginPath();
      g.moveTo(sx - 5, sz);
      g.lineTo(sx, sz - 46 * up);
      g.lineTo(sx + 5, sz);
      g.closePath();
      g.fill();
      g.stroke();
    } else {
      g.fillStyle = '#1b1822';
      g.beginPath();
      g.arc(sx, sz, 3, 0, Math.PI * 2);
      g.fill();
    }
  }
  g.restore();
}

/** 滾木：預告時在畫面右邊那條路線上閃「！」，之後滾過去 */
function drawLog(g, h, x, frame) {
  if (h.state === 'warn') {
    if (Math.floor(frame / 8) % 2) return;
    g.save();
    g.translate(x - 10, h.z - 40);
    g.fillStyle = '#ffd23f';
    g.strokeStyle = INK;
    g.lineWidth = 3;
    g.beginPath();
    g.moveTo(0, -30);
    g.lineTo(26, 16);
    g.lineTo(-26, 16);
    g.closePath();
    g.fill();
    g.stroke();
    g.fillStyle = INK;
    g.font = `900 28px "Noto Sans TC", sans-serif`;
    g.textAlign = 'center';
    g.textBaseline = 'middle';
    g.fillText('!', 0, 2);
    g.beginPath();
    g.moveTo(-40, 30);
    g.lineTo(-60, 40);
    g.lineTo(-40, 50);
    g.fill();
    g.restore();
    return;
  }
  g.save();
  g.translate(x, h.z);
  g.fillStyle = 'rgba(20,20,40,0.25)';
  g.beginPath();
  g.ellipse(0, 2, 60, 9, 0, 0, Math.PI * 2);
  g.fill();
  // 圖：1536×160 四格（每格 384×160），圓木中心 (192, 80)、長 340 px＝112 單位；格子跟著滾動換
  const img = art('log');
  if (img) {
    g.restore();
    drawArt(g, img, x, h.z - 24, { w: 384, ax: 192, ay: 80, scale: 112 / 340, frames: 4, frame: Math.floor(frame / 3) % 4 });
    return;
  }
  g.translate(0, -24);
  g.strokeStyle = INK;
  g.lineWidth = 2.5;
  g.fillStyle = '#8a5a32';
  g.fillRect(-56, -22, 112, 44);
  g.strokeRect(-56, -22, 112, 44);
  // 樹皮紋路跟著滾動
  g.strokeStyle = '#5e3d1f';
  for (let k = 0; k < 4; k++) {
    const lx = ((k * 30 - frame * 6.5) % 112 + 112) % 112 - 56;
    g.beginPath();
    g.moveTo(lx, -20);
    g.lineTo(lx + 8, 20);
    g.stroke();
  }
  g.fillStyle = '#c9965e';
  g.strokeStyle = INK;
  g.beginPath();
  g.ellipse(-56, 0, 10, 22, 0, 0, Math.PI * 2);
  g.fill();
  g.stroke();
  g.strokeStyle = '#8a5a32';
  g.beginPath();
  g.ellipse(-56, 0, 5, 12, 0, 0, Math.PI * 2);
  g.stroke();
  g.restore();
}

export function drawHazard(g, h, cam, frame) {
  const x = h.x - cam;
  if (h.kind === 'geyser') drawGeyser(g, h, x, frame);
  else if (h.kind === 'spikes') drawSpikes(g, h, x, frame);
  else if (h.kind === 'log') drawLog(g, h, x, frame);
}

/** 地上的機關（噴口、尖刺）要畫在人的後面；滾木、岩漿柱跟人一起依前後排 */
export function hazardDepth(h) {
  if (h.kind === 'log') return h.z;
  return h.state === 'active' ? h.z : h.z - 1000;
}
