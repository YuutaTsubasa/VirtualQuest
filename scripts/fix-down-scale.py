#!/usr/bin/env python3
"""倒地（down）那一格的放大倍數：倒地圖為了塞進格子、身體又畫得比較修長，Q 版的人躺下來頭只剩站著的一半多，
看起來整個人小了一號。這裡讓「躺下來的身長」約是站姿身高的 1.25 倍（頭大約回到站姿的 8 成）。

用法：python scripts/fix-down-scale.py [--style <畫風>] [--write]
  不加 --write 只印出結果。角色、雜兵、魔王都算；倒地本來就很長的（大螃蟹）不動。
  接了新的動作圖之後跑一次（measure-head-scale.py 不量 down，躺著的頭比對不準）。
"""
import json
import os
import sys

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SHEETS = os.path.join(ROOT, 'src', 'lib', 'game', 'sheets.json')
TARGET = 1.25
MAX = 1.45

args = sys.argv[1:]
WRITE = '--write' in args
STYLE = args[args.index('--style') + 1] if '--style' in args else None
ALL = json.load(open(SHEETS, encoding='utf-8'))


def box(path, cell, cols, frames, f):
    """這一格不透明部分的寬、高（格子的比例）"""
    i = frames.index(f)
    a = Image.open(path).convert('RGBA').getchannel('A')
    a = a.crop(((i % cols) * cell, (i // cols) * cell, (i % cols + 1) * cell, (i // cols + 1) * cell))
    b = a.point(lambda v: 255 if v > 128 else 0).getbbox()
    return (b[2] - b[0]) / cell, (b[3] - b[1]) / cell


changed = 0
for group in ('characters', 'enemies', 'bosses'):
    for key, base in ALL[group].items():
        if 'down' not in base['frames']:
            continue
        if STYLE:
            entry = ALL.get('styles', {}).get(STYLE, {}).get(group, {}).get(key)
            if not entry:
                continue
            path = os.path.join(ROOT, 'public', 'styles', STYLE, base['src'])
            cell = entry['cell']
        else:
            entry = base
            path = os.path.join(ROOT, 'public', base['src'])
            cell = base['cell']
        cols = base.get('cols', len(base['frames']))
        fix = entry.setdefault('fix', {})
        _, idle_h = box(path, cell, cols, base['frames'], 'idle')
        down_w, _ = box(path, cell, cols, base['frames'], 'down')
        ratio = down_w / (idle_h * fix.get('idle', 1))
        k = round(min(MAX, max(1.0, TARGET / ratio)), 3)
        print(f'{STYLE or "anime90":8} {group:10} {key:15} 躺下身長／站高 {ratio:.2f} → down ×{k}')
        if k > 1.02:
            fix['down'] = k
        else:
            fix.pop('down', None)
        if not fix:
            entry.pop('fix')
        changed += 1

if WRITE:
    json.dump(ALL, open(SHEETS, 'w', encoding='utf-8'), ensure_ascii=False, indent=2)
print(f'{changed} 張圖集{"已寫進 sheets.json" if WRITE else "（加 --write 寫入）"}')
