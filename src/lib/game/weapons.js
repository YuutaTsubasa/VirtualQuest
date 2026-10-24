// 武器的樣子（程式畫的；之後請美術畫成圖再換掉）。
// 遊戲裡只有兩類（crates/quest-core 的 ItemKind::Bat 鈍器、ItemKind::Blade 刀類），每一大關長得不一樣。
// 畫的座標：握把在 (0,0)，武器往 +x 伸出去；angle 是弧度（0 ＝ 往前、負的往上）。

// 每一大關（stages.js 的 WORLDS 順序）的鈍器、刀類
const LOOKS = [
  // 白色都市：鐵管、小刀
  { bat: { kind: 'pipe', color: '#9aa3ad', dark: '#5d646d' }, blade: { kind: 'knife', color: '#dfe5ea', hilt: '#2b2b33' } },
  // 草原森林：木棍、手斧
  { bat: { kind: 'club', color: '#9a6a3a', dark: '#5e3d1f' }, blade: { kind: 'hatchet', color: '#c9cfd4', hilt: '#7a5230' } },
  // 水下世界：船槳、珊瑚短刀
  { bat: { kind: 'oar', color: '#b8874e', dark: '#7a5530' }, blade: { kind: 'knife', color: '#ff8f8f', hilt: '#2f6f8f' } },
  // 冰山：冰棍、冰柱
  { bat: { kind: 'club', color: '#bfe9ff', dark: '#6fb3d9' }, blade: { kind: 'icicle', color: '#dff6ff', hilt: '#8fcbe8' } },
  // 火炎礦坑：十字鎬、黃銅短刀
  { bat: { kind: 'pick', color: '#8a5a32', dark: '#9aa3ad' }, blade: { kind: 'knife', color: '#e0b04a', hilt: '#3b2a1e' } },
  // 邪惡城堡：狼牙棒、短劍
  { bat: { kind: 'mace', color: '#4a3a5a', dark: '#9a8fb0' }, blade: { kind: 'sword', color: '#e6e9f2', hilt: '#6a3a9a' } },
];

const INK = '#202329';

/** 畫一把武器。s：大小倍數 */
export function drawWeapon(g, type, world, x, y, angle = 0, s = 1) {
  const look = (LOOKS[world] ?? LOOKS[0])[type === 'blade' ? 'blade' : 'bat'];
  g.save();
  g.translate(x, y);
  g.rotate(angle);
  g.scale(s, s);
  g.lineJoin = 'round';
  g.lineWidth = 2;
  g.strokeStyle = INK;
  const rect = (x0, y0, w, h, fill) => {
    g.fillStyle = fill;
    g.fillRect(x0, y0, w, h);
    g.strokeRect(x0, y0, w, h);
  };
  switch (look.kind) {
    case 'pipe':
      rect(-6, -3, 56, 6, look.color);
      rect(44, -4, 8, 8, look.dark);
      break;
    case 'club':
      g.fillStyle = look.color;
      g.beginPath();
      g.moveTo(-6, -3);
      g.lineTo(34, -5);
      g.quadraticCurveTo(54, -9, 54, 0);
      g.quadraticCurveTo(54, 9, 34, 5);
      g.lineTo(-6, 3);
      g.closePath();
      g.fill();
      g.stroke();
      rect(-6, -3, 12, 6, look.dark);
      break;
    case 'oar':
      rect(-6, -2.5, 40, 5, look.color);
      g.fillStyle = look.color;
      g.beginPath();
      g.ellipse(44, 0, 13, 7, 0, 0, Math.PI * 2);
      g.fill();
      g.stroke();
      break;
    case 'pick':
      rect(-6, -2.5, 50, 5, look.color);
      g.fillStyle = look.dark;
      g.beginPath();
      g.moveTo(40, -18);
      g.quadraticCurveTo(52, 0, 40, 18);
      g.lineTo(46, 0);
      g.closePath();
      g.fill();
      g.stroke();
      break;
    case 'mace':
      rect(-6, -2.5, 40, 5, look.color);
      g.fillStyle = look.dark;
      for (let k = 0; k < 6; k++) {
        const a = (k / 6) * Math.PI * 2;
        g.beginPath();
        g.moveTo(44 + Math.cos(a) * 8, Math.sin(a) * 8);
        g.lineTo(44 + Math.cos(a) * 14, Math.sin(a) * 14);
        g.stroke();
      }
      g.beginPath();
      g.arc(44, 0, 9, 0, Math.PI * 2);
      g.fill();
      g.stroke();
      break;
    case 'hatchet':
      rect(-6, -2.5, 34, 5, look.hilt);
      g.fillStyle = look.color;
      g.beginPath();
      g.moveTo(22, -3);
      g.lineTo(34, -12);
      g.quadraticCurveTo(40, 0, 34, 10);
      g.lineTo(22, 3);
      g.closePath();
      g.fill();
      g.stroke();
      break;
    case 'icicle':
      g.fillStyle = look.color;
      g.beginPath();
      g.moveTo(-6, -5);
      g.lineTo(38, 0);
      g.lineTo(-6, 5);
      g.closePath();
      g.fill();
      g.stroke();
      break;
    case 'sword':
      rect(-6, -2.5, 12, 5, look.hilt);
      rect(5, -6, 4, 12, look.hilt);
      g.fillStyle = look.color;
      g.beginPath();
      g.moveTo(9, -3);
      g.lineTo(40, -3);
      g.lineTo(46, 0);
      g.lineTo(40, 3);
      g.lineTo(9, 3);
      g.closePath();
      g.fill();
      g.stroke();
      break;
    default: {
      // 小刀
      rect(-6, -2.5, 12, 5, look.hilt);
      g.fillStyle = look.color;
      g.beginPath();
      g.moveTo(6, -4);
      g.lineTo(28, -2);
      g.lineTo(32, 1);
      g.lineTo(6, 4);
      g.closePath();
      g.fill();
      g.stroke();
    }
  }
  g.restore();
}

/**
 * 手上拿武器的位置與角度（面向右；遊戲座標、相對於腳底）。
 * 動作圖沒有拿武器的版本，所以武器畫在角色前面、跟著動作換角度。
 * 回傳 null ＝ 這個動作不畫（例如放必殺、被抓、丟出去）
 */
export function heldPose(p) {
  const bat = p.weapon === 'bat';
  if (p.act === 'attack' && (p.mv === 'bat' || p.mv === 'blade')) {
    if (p.mv === 'bat') {
      // 鈍器：起手舉到後上方（7 格），揮下去（5 格），停在前下方
      const k = Math.min(1, Math.max(0, (p.t - 5) / 7));
      return { x: 36, y: -92, angle: -2.3 + k * 2.9, swing: k > 0 && k < 1 ? k : 0 };
    }
    // 刀：往前刺
    return { x: p.t < 4 ? 30 : 52, y: -96, angle: p.t < 4 ? -0.5 : 0, swing: 0 };
  }
  switch (p.act) {
    case 'stand':
    case 'walk':
      return { x: 22, y: -80, angle: bat ? -1.15 : -0.7 };
    case 'run':
      return { x: 26, y: -82, angle: bat ? -0.9 : -0.4 };
    case 'jump':
      return { x: 24, y: -96, angle: -0.9 };
    case 'hurt':
    case 'rise':
      return { x: 14, y: -78, angle: -1.5 };
    case 'attack':
      // 撿起來那一下、或其他收招：放低
      return { x: 26, y: -70, angle: -0.4 };
    default:
      return null;
  }
}
