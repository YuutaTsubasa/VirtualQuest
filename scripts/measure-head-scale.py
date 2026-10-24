#!/usr/bin/env python3
"""量每一格角色動作圖的「頭」跟 idle 的頭比起來多大，修正 sheets.json 的 fix，讓動作時人物不會忽大忽小。

用法：python scripts/measure-head-scale.py [--write] [角色 id ...]
  不加 --write 只印出結果；加了就把差太多的格子寫進 src/lib/game/sheets.json 的 fix。

做法：把 idle 的頭（含頭髮，人物最上面 30%）當樣板，縮放、旋轉、左右翻去每一格的上半部找最像的位置，
找到的縮放倍數 × 目前的 fix ＝ 畫面上這格的頭是 idle 的幾倍。
同一張圖集裡 walk1／walk2 的誤差大約 ±5%，所以只修差 7% 以上、而且比對分數夠高的格子。
可以重複執行：修過的格子再量會接近 1，不會再改。接了新的動作圖之後跑一次。
"""
import json, os, sys
import cv2
import numpy as np
from PIL import Image

root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SHEETS = os.path.join(root, 'src', 'lib', 'game', 'sheets.json')
WRITE = '--write' in sys.argv
ONLY = [a for a in sys.argv[1:] if not a.startswith('--')]
ALL = json.load(open(SHEETS, encoding='utf-8'))
S = ALL['characters']
MIN_SCORE = 0.55
MIN_DIFF = 0.07
SKIP = {'idle', 'down', 'fall'}  # 躺著的頭轉 90 度，另外看


def cell(im, s, f):
    c = s['cell']; cols = s['cols']; i = s['frames'].index(f)
    x, y = (i % cols) * c, (i // cols) * c
    a = np.array(im.crop((x, y, x + c, y + c)).resize((c // 2, c // 2), Image.LANCZOS))
    return a


def prep(a):
    rgb = a[..., :3].astype(np.float32)
    al = (a[..., 3] > 128).astype(np.float32)
    g = cv2.cvtColor(rgb, cv2.COLOR_RGB2GRAY) * al + 128 * (1 - al)
    return g, al


changed = []
for cid, s in S.items():
    if ONLY and cid not in ONLY:
        continue
    im = Image.open(f'{root}/public/sprites/{cid}.webp').convert('RGBA')
    fix = s.get('fix', {})
    idle = cell(im, s, 'idle')
    g0, a0 = prep(idle)
    ys, xs = np.nonzero(a0)
    top, bot = ys.min(), ys.max()
    hh = int((bot - top) * 0.30)
    band = a0[top:top + hh]
    bx = np.nonzero(band.any(0))[0]
    tpl = g0[top:top + hh, bx.min():bx.max() + 1]
    msk = a0[top:top + hh, bx.min():bx.max() + 1]
    for f in s['frames']:
        if f in SKIP:
            continue
        g, al = prep(cell(im, s, f))
        # 只在人物上半部找頭（省時間，也不會把手腳當成頭）
        fy = np.nonzero(al.any(1))[0]
        g = g[max(0, fy.min() - 4): fy.min() + int((fy.max() - fy.min()) * 0.75) + 4]
        best = (-1, 1, 0)
        for flip in (False, True):
            t0 = cv2.flip(tpl, 1) if flip else tpl
            m0 = cv2.flip(msk, 1) if flip else msk
            for rot in (-24, -12, 0, 12, 24):
                for sc in np.arange(0.72, 1.40, 0.025):
                    w, h = int(t0.shape[1] * sc), int(t0.shape[0] * sc)
                    if w >= g.shape[1] or h >= g.shape[0]:
                        continue
                    t = cv2.resize(t0, (w, h)); m = cv2.resize(m0, (w, h))
                    if rot:
                        M = cv2.getRotationMatrix2D((w / 2, h / 2), rot, 1)
                        t = cv2.warpAffine(t, M, (w, h), borderValue=128)
                        m = cv2.warpAffine(m, M, (w, h), borderValue=0)
                    r = cv2.matchTemplate(g, t, cv2.TM_CCOEFF_NORMED, mask=m)
                    r = np.nan_to_num(r, nan=-1, posinf=-1, neginf=-1)
                    v = float(r.max())
                    if v > best[0]:
                        best = (v, float(sc), rot)
        shown = best[1] * fix.get(f, 1)
        mark = ''
        if best[0] >= MIN_SCORE and abs(shown - 1) >= MIN_DIFF:
            new = round(min(1.3, max(0.8, fix.get(f, 1) / shown)), 3)
            mark = f'→ fix {fix.get(f, 1)} → {new}'
            if new == 1:
                fix.pop(f, None)
            else:
                fix[f] = new
            changed.append((cid, f))
        print(f'{cid:9} {f:9} 頭 ×{shown:.2f}（分數 {best[0]:.2f}）{mark}', flush=True)
    if fix:
        s['fix'] = fix
if WRITE and changed:
    json.dump(ALL, open(SHEETS, 'w', encoding='utf-8'), ensure_ascii=False, indent=2)
print(f'{len(changed)} 格{"已寫進 sheets.json" if WRITE else "要修（加 --write 寫入）"}')
