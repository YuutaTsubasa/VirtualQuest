# 圖示：從 Logo 取紅色「闖」字，放在米色圓角方塊上；另外做分享卡片。
# 用法：python scripts/make-icons.py <repo> <輸出資料夾>
#   產出 app-icon.png（1024，給 npx tauri icon）、favicon.ico／favicon.png／apple-touch-icon.png、og-image.jpg
import sys
from PIL import Image, ImageDraw, ImageFilter, ImageChops
repo, out = sys.argv[1], sys.argv[2]
logo = Image.open(f'{repo}/public/ui/logo.webp').convert('RGBA')
reg = logo.crop((40, 410, 600, 880))
px = reg.load(); w, h = reg.size
red = Image.new('L', reg.size, 0); rp = red.load()
for y in range(h):
    for x in range(w):
        r, g, b, a = px[x, y]
        if a > 128 and r > 170 and g < 140 and b < 120 and r - g > 80:
            rp[x, y] = 255
red = red.filter(ImageFilter.MaxFilter(3)).filter(ImageFilter.MinFilter(3))
red = red.crop(red.getbbox())
S = 1024
bg = Image.new('RGBA', (S, S), (0, 0, 0, 0))
mask = Image.new('L', (S, S), 0)
ImageDraw.Draw(mask).rounded_rectangle((0, 0, S - 1, S - 1), radius=200, fill=255)
tile = Image.new('RGBA', (S, S), (246, 238, 220, 255))
d = ImageDraw.Draw(tile)
d.rounded_rectangle((28, 28, S - 29, S - 29), radius=176, outline=(26, 24, 28, 255), width=36)
bg.paste(tile, (0, 0), mask)
gw = 700
g = red.resize((gw, round(red.height * gw / red.width)), Image.LANCZOS)
if g.height > 720:
    gh = 720; g = red.resize((round(red.width * gh / red.height), gh), Image.LANCZOS)
ox, oy = (S - g.width) // 2, (S - g.height) // 2 + 6
full = Image.new('L', (S, S), 0); full.paste(g, (ox, oy))
outline = full.filter(ImageFilter.MaxFilter(41))
shadow = Image.new('L', (S, S), 0); shadow.paste(outline, (16, 20))
ink = Image.new('RGBA', (S, S), (26, 24, 28, 255))
bg = Image.composite(ink, bg, ImageChops.multiply(shadow, mask))
bg = Image.composite(ink, bg, outline)
bg = Image.composite(Image.new('RGBA', (S, S), (233, 75, 50, 255)), bg, full)
bg.save(f'{out}/app-icon.png')
bg.resize((192, 192), Image.LANCZOS).save(f'{out}/favicon.png')
flat = Image.new('RGBA', (S, S), (246, 238, 220, 255)); flat.alpha_composite(bg)
flat.convert('RGB').resize((180, 180), Image.LANCZOS).save(f'{out}/apple-touch-icon.png')
bg.save(f'{out}/favicon.ico', sizes=[(16, 16), (32, 32), (48, 48)])

# 分享卡片 1200×630：標題主視覺鋪滿，左邊放 Logo
scene = Image.open(f'{repo}/public/ui/title-scene.webp').convert('RGBA')
W, H = 1200, 630
k = max(W / scene.width, H / scene.height)
scene = scene.resize((round(scene.width * k), round(scene.height * k)), Image.LANCZOS)
card = scene.crop((0, (scene.height - H) // 2, W, (scene.height - H) // 2 + H))
lw = 440
lg = logo.resize((lw, round(logo.height * lw / logo.width)), Image.LANCZOS)
card.alpha_composite(lg, (24, (H - lg.height) // 2))
card.convert('RGB').save(f'{out}/og-image.jpg', quality=88)
