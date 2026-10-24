#!/usr/bin/env python3
"""把 ChatGPT 交的 90 年代美術包（art-inbox/<主題>/ 或 .zip）接進遊戲。

用法：python scripts/import-quest-art.py art-inbox/anime90-trial
      python scripts/import-quest-art.py art-inbox/pc98-characters.zip --style pc98

--style（pc98 msdos taiwan-print gameboy modern-paint）：別的畫風。圖放 public/styles/<風格>/，
  圖集的格子順序、欄數跟 anime90 完全一樣（遊戲的打擊判定、動作都不用改），只有格子大小與補償倍數
  記在 sheets.json 的 styles.<風格>。像素風格用最近鄰縮回邏輯解析度（pc98／msdos 256、gameboy 128）、
  無損 WebP，像素不會糊。這次沒交、這個畫風也還沒有的格子先借 anime90 的那一格（會印出來）。

交件的結構見 art-inbox/README.md：
  characters/<id>/sprites/<動作>.png        512×512，腳底 (256, 496)
  enemies/<大關>/<種類>/<動作>.png           512×512，腳底 (256, 496)
  bosses/<大關>/<動作>.png                   768×768，腳底 (384, 744)
  environment/<大關>-<小關>/{far,mid,ground}.png   1920×1080
  props/<大關>-crate.png、props/items/*.png、props/shots/*.png
  ui/*.png（Logo 不接：介面改用 UI Concept v1 的 Logo）

產出（都是 WebP）：
  public/sprites/<id>.webp                   8×2 圖集（每格 400），格子順序見 src/lib/game/sheets.json
  public/enemies/<大關>/<種類>.webp          橫排（每格 400）
  public/bosses/<大關>.webp                  橫排（每格 600，跟角色同比例尺）
  public/stages/<大關>-<小關>/{far,mid,ground}.webp
  public/props/...
並更新 src/lib/game/sheets.json（畫面用）與 src/lib/style-assets.json（預先載入用）。

圖的 alpha：生圖工具交的人物大多是 253/255，接的時候把 ≥240 的拉成全不透明，免得角色微微透光。

寬的姿勢（高踢、敵人出手、衝撞）為了塞進格子會整個縮小（站著 400px、踢腿只剩 280～350px）。
交件包的 layout-metrics.json 有每格的實際人物高度：這些格子記一個放大倍數（fix），
畫面會以腳底為中心放大回來，踢腿時人不會突然變小。
"""

import json
import os
import shutil
import sys
import tempfile
import zipfile

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PUBLIC = os.path.join(ROOT, 'public')
SHEETS = os.path.join(ROOT, 'src', 'lib', 'game', 'sheets.json')
ANIME90_PUBLIC = PUBLIC
STYLES = ['pc98', 'msdos', 'taiwan-print', 'gameboy', 'modern-paint']
# 像素風格：交件是邏輯解析度 × 整數倍，接的時候縮回邏輯解析度（角色 512 → 256／128，魔王 768 → 384／192）
PIXEL_DIV = {'pc98': 2, 'msdos': 2, 'gameboy': 4}
STYLE = None  # main() 依 --style 設定
MANIFEST = os.path.join(ROOT, 'src', 'lib', 'style-assets.json')

CHARACTER_FRAMES = ['idle', 'walk1', 'walk2', 'punch1', 'punch2', 'kick', 'jump', 'jumpkick',
                    'special', 'grab', 'knee', 'throw', 'hurt', 'fall', 'down', 'win',
                    # 跑步與衝刺攻擊（後來加的，requests/2026-10-24-0910-anime90-run-frames.md）
                    'run1', 'run2', 'dash',
                    # 走路補成四格：兩格收腳（requests/2026-10-24-1500-anime90-walk-down-fix.md）
                    'walk3', 'walk4']
ENEMY_FRAMES = ['idle', 'walk1', 'walk2', 'windup', 'attack', 'hurt', 'fall', 'down', 'charge', 'walk3', 'walk4']
BOSS_FRAMES = ['idle', 'walk1', 'walk2', 'windup', 'slam', 'hurt', 'fall', 'down', 'charge', 'leap', 'cast', 'vanish', 'walk3', 'walk4']
CELL = 400
# 哪些格子要依實際高度放大回來：目標高度＝這張圖集 idle 的高度 × 比例；最多放大幾倍
FIX_CHARACTER = {'kick': 0.925, 'dash': 0.85}
# charge 常常是壓肩低頭的姿勢，只補到站立高度的 80%
FIX_ENEMY = {'attack': 0.9, 'windup': 0.95, 'charge': 0.8, 'walk1': 0.95, 'walk2': 0.95}
# 魔王的 slam／cast 常常是故意蹲低的姿勢（例如森林巨熊雙掌觸地），不能照 idle 高度放大，只補 charge
FIX_BOSS = {'charge': 0.8, 'walk1': 0.95, 'walk2': 0.95}
# 整張圖集太小（長吻、大鉗子、披風為了塞進格子而縮小）：站立 idle 應該有的大小
# 人形用高度比；大隻的（brute）用面積比，免得很寬的大螃蟹被放到比格子還大
IDLE_HEIGHT = {'character': 400, 'enemy': 380, 'boss': 650}
BRUTE_IDLE = (460, 381)
FIX_MAX = 1.45
BOSS_CELL = 600


def load(path):
    im = Image.open(path).convert('RGBA')
    a = im.getchannel('A').point(lambda v: 255 if v >= 240 else v)
    im.putalpha(a)
    return im


def save(im, rel, lossless=False):
    lossless = lossless or STYLE in PIXEL_DIV
    out = os.path.join(PUBLIC, rel)
    os.makedirs(os.path.dirname(out), exist_ok=True)
    if lossless:
        im.save(out, 'WEBP', lossless=True, method=6)
    else:
        im.save(out, 'WEBP', quality=90, method=6, alpha_quality=95)
    print('  ', rel, im.size)
    return rel.replace(os.sep, '/')


def heights(metrics, prefix):
    """layout-metrics.json 裡每格的實際人物高度：{動作: 高度}"""
    out = {}
    for path, info in (metrics.get('assets') or {}).items():
        if path.startswith(prefix) and path.endswith('.png'):
            h = info.get('actualSilhouetteHeight') or (info.get('visibleBounds') or {}).get('height')
            if h:
                out[path[len(prefix):-4]] = h
    return out


def fixes(names, hs, table):
    out = {}
    idle = hs.get('idle')
    if not idle:
        return out
    for name, ratio in table.items():
        if name in names and hs.get(name):
            k = min(FIX_MAX, max(1.0, idle * ratio / hs[name]))
            if k > 1.02:
                out[name] = round(k, 3)
    return out


def sheet_scale(metrics, prefix, kind):
    info = (metrics.get('assets') or {}).get(prefix + 'idle.png') or {}
    b = info.get('visibleBounds') or {}
    h = info.get('actualSilhouetteHeight') or b.get('height')
    w = b.get('width')
    if not h:
        return 1.0
    if kind == 'brute' and w:
        k = ((BRUTE_IDLE[0] * BRUTE_IDLE[1]) / (h * w)) ** 0.5
    else:
        k = IDLE_HEIGHT['boss' if kind == 'boss' else 'character' if kind == 'character' else 'enemy'] / h
    k = min(FIX_MAX, max(1.0, k))
    return round(k, 3) if k > 1.03 else 1.0


def old_cells(entry):
    """已經接過的圖集拆回一格一格：{動作: 圖}（補交幾格時，其他格沿用原本的）"""
    if not entry:
        return {}
    path = os.path.join(PUBLIC, entry['src'])
    if not os.path.exists(path):
        return {}
    atlas = Image.open(path).convert('RGBA')
    c, cols = entry['cell'], entry['cols']
    return {name: atlas.crop(((i % cols) * c, (i // cols) * c, (i % cols + 1) * c, (i // cols + 1) * c))
            for i, name in enumerate(entry['frames'])}


def kept_fix(before, names, folder):
    """原本圖集裡、這次沒有重畫的格子的放大倍數"""
    redrawn = {f for f in names if os.path.exists(os.path.join(folder, f + '.png'))}
    return {f: v for f, v in (before or {}).get('fix', {}).items() if f not in redrawn}


def strip(frames, folder, cell, cols=None, keep=None, exact=False):
    """把有的格子排成圖集；回傳（圖, 實際有的格子名, 欄數）。
    keep：原本圖集裡的格子，這次沒交的就沿用（例如只補交跑步 3 格）
    exact：別的畫風——格子固定照 frames 的順序（anime90 那張圖集的順序），沒有的從 keep 借"""
    keep = keep or {}
    new = {f for f in frames if os.path.exists(os.path.join(folder, f + '.png'))}
    if not new:
        return None, [], 0
    names = list(frames) if exact else [f for f in frames if f in new or f in keep]
    resample = Image.NEAREST if STYLE in PIXEL_DIV else Image.LANCZOS
    cols = cols or len(names)
    rows = (len(names) + cols - 1) // cols
    sheet = Image.new('RGBA', (cell * cols, cell * rows), (0, 0, 0, 0))
    for i, name in enumerate(names):
        if name in new:
            im = load(os.path.join(folder, name + '.png')).resize((cell, cell), resample)
        elif name in keep:
            im = keep[name].resize((cell, cell), resample)
        else:
            continue
        sheet.paste(im, ((i % cols) * cell, (i // cols) * cell))
    return sheet, names, cols


def style_atlas(sheets, group, key, folder, metrics, prefix, kind, table, written):
    """別的畫風的一張圖集：格子順序、欄數照 anime90；格子大小、倍數記在 sheets.styles.<風格>"""
    base = sheets[group].get(key)
    if not base:
        print(f'  ！{group}/{key}：anime90 還沒有這張圖集，先接 anime90')
        return
    styles = sheets.setdefault('styles', {}).setdefault(STYLE, {}).setdefault(group, {})
    before = styles.get(key)
    div = PIXEL_DIV.get(STYLE, 1)
    # 像素風格：交件 512／768 → 邏輯 256／128、384／192；其他照 anime90 的格子大小
    cell = (512 if group != 'bosses' else 768) // div if div > 1 else base['cell']
    # 這個畫風之前已經接過的格子（補交幾格時沿用）
    mine = old_cells_at(os.path.join(PUBLIC, base['src']), before['cell'], base['cols'], base['frames']) if before else {}
    anime = old_cells_at(os.path.join(ANIME90_PUBLIC, base['src']), base['cell'], base['cols'], base['frames'])
    borrowed = [f for f in base['frames'] if f not in mine and not os.path.exists(os.path.join(folder, f + '.png'))]
    sheet, names, cols = strip(base['frames'], folder, cell, cols=base['cols'], keep={**anime, **mine}, exact=True)
    if not sheet:
        return
    if borrowed:
        print(f'  ！{group}/{key} 還沒有 {borrowed}，先借 anime90 的')
    written.append(save(sheet, base['src']))
    entry = {'cell': cell}
    fix = {**kept_fix(before, names, folder), **fixes(names, heights(metrics, prefix), table)}
    if fix:
        entry['fix'] = fix
    k = sheet_scale(metrics, prefix, kind)
    if k > 1:
        entry['scale'] = k
    elif (before or {}).get('scale'):
        entry['scale'] = before['scale']
    styles[key] = entry


def pixel_down(im, div=None):
    """像素風格：交件是邏輯解析度 × 整數倍，縮回邏輯解析度（最近鄰，像素不會糊）"""
    div = div or PIXEL_DIV.get(STYLE, 1)
    if STYLE not in PIXEL_DIV or div <= 1:
        return im
    return im.resize((im.width // div, im.height // div), Image.NEAREST)


def old_cells_at(path, cell, cols, frames):
    """某張圖集拆回一格一格（別的畫風用：格子大小跟 anime90 不一樣）"""
    if not os.path.exists(path):
        return {}
    atlas = Image.open(path).convert('RGBA')
    return {name: atlas.crop(((i % cols) * cell, (i // cols) * cell, (i % cols + 1) * cell, (i // cols + 1) * cell))
            for i, name in enumerate(frames)}


def main(src):
    tmp = None
    if src.endswith('.zip'):
        tmp = tempfile.mkdtemp()
        with zipfile.ZipFile(src) as z:
            for n in z.namelist():
                if n.startswith('/') or '..' in n.split('/'):
                    sys.exit(f'壓縮檔路徑不安全：{n}')
            z.extractall(tmp)
        entries = os.listdir(tmp)
        src = os.path.join(tmp, entries[0]) if len(entries) == 1 and os.path.isdir(os.path.join(tmp, entries[0])) else tmp

    metrics_path = os.path.join(src, 'layout-metrics.json')
    metrics = json.load(open(metrics_path, encoding='utf-8')) if os.path.exists(metrics_path) else {}
    sheets = json.load(open(SHEETS, encoding='utf-8')) if os.path.exists(SHEETS) else {}
    for k in ['characters', 'enemies', 'bosses', 'stages', 'props', 'items', 'shots']:
        sheets.setdefault(k, {} if k in ('characters', 'enemies', 'bosses') else [])
    written = []

    chars = os.path.join(src, 'characters')
    for cid in sorted(os.listdir(chars)) if os.path.isdir(chars) else []:
        folder = os.path.join(chars, cid, 'sprites')
        if STYLE:
            style_atlas(sheets, 'characters', cid, folder, metrics, f'characters/{cid}/sprites/', 'character', FIX_CHARACTER, written)
            continue
        before = sheets['characters'].get(cid)
        sheet, names, cols = strip(CHARACTER_FRAMES, folder, CELL, cols=8, keep=old_cells(before))
        if sheet:
            written.append(save(sheet, f'sprites/{cid}.webp'))
            sheets['characters'][cid] = {'src': f'sprites/{cid}.webp', 'frames': names, 'cols': cols, 'cell': CELL}
            # 補交幾格時，原本那些格的放大倍數也要留著；這次重畫的格子舊的倍數就不算了
            # （頭的大小用 scripts/measure-head-scale.py --write 重新量）
            fix = {**kept_fix(before, names, folder), **fixes(names, heights(metrics, f'characters/{cid}/sprites/'), FIX_CHARACTER)}
            if fix:
                sheets['characters'][cid]['fix'] = fix
            if (before or {}).get('scale'):
                sheets['characters'][cid]['scale'] = before['scale']
            k = sheet_scale(metrics, f'characters/{cid}/sprites/', 'character')
            if k > 1:
                sheets['characters'][cid]['scale'] = k

    enemies = os.path.join(src, 'enemies')
    for world in sorted(os.listdir(enemies)) if os.path.isdir(enemies) else []:
        for kind in sorted(os.listdir(os.path.join(enemies, world))):
            folder = os.path.join(enemies, world, kind)
            key = f'{world}/{kind}'
            if STYLE:
                style_atlas(sheets, 'enemies', key, folder, metrics, f'enemies/{world}/{kind}/', kind, FIX_ENEMY, written)
                continue
            before = sheets['enemies'].get(key)
            # 補交幾格（例如只補走路 2 格）時，原本的格子、倍數都留著
            sheet, names, cols = strip(ENEMY_FRAMES, folder, CELL, keep=old_cells(before))
            if sheet:
                rel = f'enemies/{world}/{kind}.webp'
                written.append(save(sheet, rel))
                sheets['enemies'][key] = {'src': rel, 'frames': names, 'cols': cols, 'cell': CELL}
                fix = {**kept_fix(before, names, folder), **fixes(names, heights(metrics, f'enemies/{world}/{kind}/'), FIX_ENEMY)}
                if fix:
                    sheets['enemies'][key]['fix'] = fix
                if (before or {}).get('scale'):
                    sheets['enemies'][key]['scale'] = before['scale']
                k = sheet_scale(metrics, f'enemies/{world}/{kind}/', kind)
                if k > 1:
                    sheets['enemies'][key]['scale'] = k

    bosses = os.path.join(src, 'bosses')
    for world in sorted(os.listdir(bosses)) if os.path.isdir(bosses) else []:
        folder = os.path.join(bosses, world)
        if STYLE:
            style_atlas(sheets, 'bosses', world, folder, metrics, f'bosses/{world}/', 'boss', FIX_BOSS, written)
            continue
        before = sheets['bosses'].get(world)
        sheet, names, cols = strip(BOSS_FRAMES, folder, BOSS_CELL, cols=6, keep=old_cells(before))
        if sheet:
            rel = f'bosses/{world}.webp'
            written.append(save(sheet, rel))
            sheets['bosses'][world] = {'src': rel, 'frames': names, 'cols': cols, 'cell': BOSS_CELL}
            fix = {**kept_fix(before, names, folder), **fixes(names, heights(metrics, f'bosses/{world}/'), FIX_BOSS)}
            if fix:
                sheets['bosses'][world]['fix'] = fix
            if (before or {}).get('scale'):
                sheets['bosses'][world]['scale'] = before['scale']
            k = sheet_scale(metrics, f'bosses/{world}/', 'boss')
            if k > 1:
                sheets['bosses'][world]['scale'] = k

    env = os.path.join(src, 'environment')
    for stage in sorted(os.listdir(env)) if os.path.isdir(env) else []:
        layers = [l for l in ('far', 'mid', 'ground') if os.path.exists(os.path.join(env, stage, l + '.png'))]
        if len(layers) < 3:
            print(f'  ！{stage} 只有 {layers}，三層都到了才接')
            continue
        for layer in layers:
            im = Image.open(os.path.join(env, stage, layer + '.png'))
            im = im.convert('RGB') if layer == 'far' else load(os.path.join(env, stage, layer + '.png'))
            im = pixel_down(im, 4)
            written.append(save(im, f'stages/{stage}/{layer}.webp'))
        if stage not in sheets['stages']:
            sheets['stages'].append(stage)

    props = os.path.join(src, 'props')
    if os.path.isdir(props):
        for f in sorted(os.listdir(props)):
            if f.endswith('.png'):
                name = f[:-4]
                written.append(save(pixel_down(load(os.path.join(props, f))), f'props/{name}.webp'))
                if name not in sheets['props']:
                    sheets['props'].append(name)
        for sub, key in (('items', 'items'), ('shots', 'shots')):
            d = os.path.join(props, sub)
            for f in sorted(os.listdir(d)) if os.path.isdir(d) else []:
                if f.endswith('.png'):
                    name = f[:-4]
                    written.append(save(pixel_down(load(os.path.join(d, f))), f'props/{sub}/{name}.webp'))
                    if name not in sheets[key]:
                        sheets[key].append(name)

    ui = os.path.join(src, 'ui')
    if STYLE and os.path.isdir(ui):
        for dirpath, _, files in os.walk(ui):
            for f in sorted(files):
                if f.endswith('.png'):
                    rel = os.path.relpath(os.path.join(dirpath, f), src)[:-4] + '.webp'
                    im = Image.open(os.path.join(dirpath, f))
                    opaque = f.startswith(('title-scene', 'mode-'))
                    im = im.convert('RGB') if opaque else load(os.path.join(dirpath, f))
                    written.append(save(pixel_down(im), rel.replace(os.sep, '/')))

    for k in ('stages', 'props', 'items', 'shots'):
        sheets[k].sort()
    json.dump(sheets, open(SHEETS, 'w', encoding='utf-8'), ensure_ascii=False, indent=2)

    manifest = json.load(open(MANIFEST, encoding='utf-8'))
    style_id = STYLE or 'anime90'
    files = set(manifest['styles'][style_id].get('files', []))
    files.update(written)
    manifest['styles'][style_id]['files'] = sorted(files)
    open(MANIFEST, 'w', encoding='utf-8').write(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    print(f'接了 {len(written)} 張圖，sheets.json 與 style-assets.json 已更新')
    if os.path.isdir(chars):
        print(f'有角色動作圖：記得跑 python scripts/measure-head-scale.py --write{" --style " + STYLE if STYLE else ""}，修正各格頭的大小')
    if tmp:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == '__main__':
    args = sys.argv[1:]
    if '--style' in args:
        i = args.index('--style')
        STYLE = args[i + 1]
        del args[i:i + 2]
        if STYLE not in STYLES:
            sys.exit(f'不認得的畫風 {STYLE}：{STYLES}')
        PUBLIC = os.path.join(ROOT, 'public', 'styles', STYLE)
    if len(args) != 1:
        sys.exit(__doc__)
    main(args[0])
