# 把 Codex 畫的原圖（raw-quest/<畫風>/...png）整理成闖天下要的尺寸、錨點、色盤，寫進 repo 的 public/。
# 規格：art-inbox/requests/2026-10-29-1200-style-gaps.md
import os
import sys
from PIL import Image, ImageFilter, ImageChops

RAW, REPO = sys.argv[1], sys.argv[2]
PIXEL = {'pc98': 2, 'msdos': 2, 'gameboy': 4}
GB = [(0x0F, 0x38, 0x0F), (0x30, 0x62, 0x30), (0x8B, 0xAC, 0x0F), (0x9B, 0xBC, 0x0F)]
COLORS = {'pc98': 16, 'msdos': 16}


def crop(im):
    im = im.convert('RGBA')
    return im.crop(im.getbbox())


def scaled(im, w=None, h=None):
    if w and h:
        k = min(w / im.width, h / im.height)
    elif w:
        k = w / im.width
    else:
        k = h / im.height
    return im.resize((max(1, round(im.width * k)), max(1, round(im.height * k))), Image.LANCZOS)


def place(canvas, im, cx, bottom=None, cy=None):
    x = round(cx - im.width / 2)
    y = round(bottom - im.height) if bottom is not None else round(cy - im.height / 2)
    y = max(0, min(canvas.height - im.height, y))
    canvas.alpha_composite(im, (x, y))
    return canvas


def finish(style, im):
    """像素畫風：縮到邏輯解析度、量化到色盤、透明度二值化"""
    f = PIXEL.get(style)
    if not f:
        return im
    small = im.resize((im.width // f, im.height // f), Image.LANCZOS)
    alpha = small.getchannel('A').point(lambda a: 255 if a >= 128 else 0)
    rgb = Image.new('RGB', small.size, (0, 0, 0))
    rgb.paste(small.convert('RGB'), mask=alpha)
    if style == 'gameboy':
        pal = Image.new('P', (1, 1))
        pal.putpalette([c for rgbv in GB for c in rgbv] + [0] * (256 - 4) * 3)
        q = rgb.quantize(palette=pal, dither=Image.Dither.NONE)
    else:
        q = rgb.quantize(colors=COLORS[style], method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    out = q.convert('RGBA')
    out.putalpha(alpha)
    return out


def hot_hole(im):
    """噴口預告：洞裡偏紅、偏暗的地方發亮（往橘黃推），岩石不變"""
    out = im.copy()
    px = out.load()
    for y in range(im.height):
        for x in range(im.width):
            r, g, b, a = px[x, y]
            if a and r > g + 25 and r > b + 25:
                px[x, y] = (min(255, r + 90), min(255, g + 70), min(255, b + 10), a)
            elif a and r + g + b < 120:
                px[x, y] = (min(255, r + 110), min(255, g + 35), b, a)
    return out


def red_holes(im):
    """尖刺預告：暗的洞透出紅光"""
    px = im.load()
    out = im.copy()
    po = out.load()
    for y in range(im.height):
        for x in range(im.width):
            r, g, b, a = px[x, y]
            if a > 0 and r + g + b < 150:
                po[x, y] = (min(255, 120 + r * 2), 20 + g // 3, 30 + b // 3, a)
    return out.filter(ImageFilter.SMOOTH) if False else out


def log_frames(im):
    """滾木：4 格，樹皮的細節往上捲（繞長軸轉）。外框、明暗、左端年輪截面不動：
    只把「原圖減掉模糊版」的細節捲動，再加回原本的模糊明暗，捲到的範圍是內縮過的木頭內部"""
    import numpy as np
    frames = []
    l, t, r, b = im.getbbox()
    cap = l + int((r - l) * 0.18)
    a = np.asarray(im.getchannel('A')).astype(np.float32) / 255
    inner = np.asarray(im.getchannel('A').filter(ImageFilter.MinFilter(9))).astype(np.float32) / 255
    inner[:, :cap] = 0
    rgb = np.asarray(im.convert('RGB')).astype(np.float32)
    blur = np.asarray(im.convert('RGB').filter(ImageFilter.GaussianBlur(6))).astype(np.float32)
    detail = rgb - blur
    # 每一欄各自在「木頭內部」那一段裡捲（圓木畫得有點斜，不能整排一起捲）
    for k in range(4):
        d = detail.copy()
        for x in range(cap, r):
            ys = np.where(inner[:, x] > 0.99)[0]
            if len(ys) < 8:
                continue
            y0, y1 = ys.min(), ys.max() + 1
            d[y0:y1, x] = np.roll(detail[y0:y1, x], -round((y1 - y0) * k / 4), axis=0)
        out = rgb * (1 - inner[..., None]) + (blur + d) * inner[..., None]
        f = Image.fromarray(np.clip(out, 0, 255).astype(np.uint8), 'RGB').convert('RGBA')
        f.putalpha(im.getchannel('A'))
        frames.append(f)
    return frames


def save(style, rel, im):
    base = os.path.join(REPO, 'public') if style == 'anime90' else os.path.join(REPO, 'public', 'styles', style)
    path = os.path.join(base, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    finish(style, im).save(path, 'WEBP', lossless=bool(PIXEL.get(style)), quality=92, method=6)
    print('寫好', style, rel)


def raw(style, rel):
    p = os.path.join(RAW, style, rel)
    return crop(Image.open(p)) if os.path.exists(p) else None


for style in ['anime90', 'pc98', 'msdos', 'taiwan-print', 'gameboy', 'modern-paint']:
    if not os.path.isdir(os.path.join(RAW, style)):
        continue
    if (im := raw(style, 'props/barrel.png')):
        save(style, 'props/barrel.webp', place(Image.new('RGBA', (256, 256)), scaled(im, w=170, h=210), 128, bottom=244))
    if (im := raw(style, 'props/hazards/geyser-idle.png')):
        hole = scaled(im, w=224, h=110)
        idle = place(Image.new('RGBA', (256, 128)), hole, 128, cy=100)
        save(style, 'props/hazards/geyser-idle.webp', idle)
        save(style, 'props/hazards/geyser-warn.webp', hot_hole(idle))
    if (im := raw(style, 'props/hazards/geyser-active.png')):
        save(style, 'props/hazards/geyser-active.webp', place(Image.new('RGBA', (256, 640)), scaled(im, w=230, h=625), 128, bottom=630))
    if (im := raw(style, 'props/hazards/spikes-idle.png')):
        plate = scaled(im, w=230)
        idle = place(Image.new('RGBA', (256, 256)), plate, 128, cy=220)
        save(style, 'props/hazards/spikes-idle.webp', idle)
        save(style, 'props/hazards/spikes-warn.webp', red_holes(idle))
        bottom = min(256, round(220 + plate.height / 2))
        if (act := raw(style, 'props/hazards/spikes-active.png')):
            # 鐵板對齊平時那張的底；尖刺最高到 y≈80（原圖的鐵板寬跟平時一樣）
            a = scaled(act, w=230)
            save(style, 'props/hazards/spikes-active.webp', place(Image.new('RGBA', (256, 256)), a if a.height <= bottom - 70 else scaled(act, h=bottom - 70), 128, bottom=bottom))
    if (im := raw(style, 'props/hazards/log.png')):
        one = place(Image.new('RGBA', (384, 160)), scaled(im, w=340, h=150), 192, cy=80)
        atlas = Image.new('RGBA', (1536, 160))
        for k, f in enumerate(log_frames(one)):
            atlas.alpha_composite(f, (k * 384, 0))
        save(style, 'props/hazards/log.webp', atlas)
    if style != 'anime90' and (im := raw(style, 'ui/logo.png')):
        save(style, 'ui/logo.webp', place(Image.new('RGBA', (1400, 560)), scaled(im, w=1360, h=540), 700, cy=280))
