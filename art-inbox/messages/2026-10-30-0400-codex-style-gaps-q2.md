# VirtualQuest Q2 原圖交件

日期：2026-10-30
工具：內建 image_gen。全部影像只透過圖片生成／圖片編輯工具繪製；交件僅複製工具 PNG 原檔，未用 Python，未裁切、縮放、量化或修改像素。未修改遊戲 repo，未執行 git。

範圍：PC-98 7 張、MS-DOS 7 張、Game Boy 6 張，共 20 張。依本次訊息不含 warn、不含 Game Boy Logo，圓木為單張原圖而非四格圖集。

已看參考：三畫風 props/city-crate.webp 與 stages/{mine-1,castle-1,forest-1}/ground.webp；public/ui/logo.webp、public/styles/gameboy/ui/logo.webp。

## 原圖限制與品質記錄

- 每次都要求透明背景。原圖的像素風是生成結果，不能保證每一 RGB 精確落在指定色票，部分填色仍有過渡色與邊緣細像素；尤其 Game Boy 四色與 MS-DOS EGA 的嚴格色值，請 Claude 整理色盤時逐像素檢查。這批不應視為已完成色盤驗證的遊戲成品。
- 三組 spikes-active 已重畫一次，提高尖刺；保留六孔與相似底座，但生成式重繪不保證與 idle 像素級重合。匯入時需檢查中心與底板寬度。
- PC-98、MS-DOS geyser-active 初版有柔光暈，已淘汰。最後改以 Game Boy 無光暈輪廓為參考重新上色，明顯柔光已消除。代價是 active 的岩石環與各自 idle 不是完全同形，且孔口投影偏深；如必須無縫切換，建議重畫／整合底座。柱高比例為視覺近似，並非精確量測。
- Game Boy geyser-active 底座較寬，柱高相對洞口寬仍不足 2.5 倍，建議優先重畫該比例。
- Logo 文字目視為「虛擬／闖天下」和「VIRTUAL QUEST」，闖字紅色。均為橫向原圖，畫布透明留白較多，Claude 可依實際內容整理。
- 所有物件無地面投影，無浮水印；未缺件。以下「可交原圖」不代表已通過嚴格色盤及錨點驗收。

## 各張檔名、提示詞與評語

### pc98/props/barrel.png

評語：可交原圖；無特別內容重畫需求。仍需 Claude 做色盤、尺寸與錨點整理。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Single game sprite for Virtual Quest, PC-98 pixel art. Upright RED gasoline barrel, yellow-black warning stripe around body and bold pictogram flame hazard symbol without text. Front view slightly above, visible elliptical top, lit upper left. Match the viewed PC98 city crate: dark indigo-purple outlines, muted metal highlights, warm vermilion reds, approximately 16 flat colors, chunky visible square pixels, sparse ordered dithering, no gradients or antialiasing. Center complete object with generous transparent margin. Actual transparent background, no ground, no cast shadow, no text, no watermark. Square canvas. Create only this one barrel.
```

### pc98/props/hazards/geyser-idle.png

評語：可交原圖；無特別內容重畫需求。仍需 Claude 做色盤、尺寸與錨點整理。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: one isolated beat-em-up game sprite for Virtual Quest. Single broad flat rocky ground vent, ellipse width 3.5 times height, dark hot interior, only two tiny angular wisps of steam. Oblique top/front view. Landscape canvas. Style: PC-98 pixel art matching viewed PC98 reference sprites and grounds: dark indigo purple contours, muted cool slate rock, warm rust ochre and vermilion highlights, about 16 flat colors. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely no gradients, no antialiasing. Light from upper left matching reference. Actual transparent background; no ground plane, no cast shadow, no scene, no watermark. Complete silhouette with transparent margin on all sides. No text except requested logo.
```

### pc98/props/hazards/geyser-active.png

評語：已多次重畫並去掉明顯柔光；不滿意：active 與 idle 岩環同形程度不足，建議要求無縫切換時重畫底座。

提示詞（最後採用版）：

```text
Recolor the reference hard-edged sprite into pc98 art. Keep its exact outlined silhouette, transparency and block pixels. PC-98 pixel art matching viewed PC98 reference sprites and grounds: dark indigo purple contours, muted cool slate rock, warm rust ochre and vermilion highlights, about 16 flat colors. Central vertical shape flat scarlet/red/yellow; rocks dark slate indigo. STRICT FLAT INK, this is colored paper shapes, it does not emit light. NO glow, no halo, no bloom, no atmospheric light. Absolutely transparent background all the way to solid contour. Narrow bottom ellipse width to 60% so vertical shape height equals 2.5 times ellipse width. Do not add backdrop or shadow.
```

### pc98/props/hazards/spikes-idle.png

評語：可交原圖；無特別內容重畫需求。仍需 Claude 做色盤、尺寸與錨點整理。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. Single wide shallow iron trap plate, oblique top/front view, one straight row of six dark empty holes, small corner rivets, slight chipped castle stone edge attached to plate only. Low flat silhouette, wide landscape canvas. PC-98 pixel art matching viewed PC98 reference sprites and grounds: dark indigo purple contours, muted cool slate rock, warm rust ochre and vermilion highlights, about 16 flat colors. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, no gradients, no antialiasing. Upper-left light. Actual transparent background, complete silhouette and margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. 
```

### pc98/props/hazards/spikes-active.png

評語：已重畫加高尖刺，目視接近需求。底座與 idle 須對位，生成式重繪存在細部差異。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. Edit the reference idle trap plate into fully extended state. Preserve exact same six holes, plate silhouette, rivets and rock trim. Raise ONE sharp triangular iron spike from each hole. Each spike must project ABOVE THE BACK EDGE of plate by a clearly visible height equal to 20% of total plate width; these must be long upright pointed spikes, not little studs hidden in holes. Height 300 pixels for a 1500-pixel-wide plate. Entire spikes inside canvas, transparent margins. PC-98 pixel art matching viewed PC98 reference sprites and grounds: dark indigo purple contours, muted cool slate rock, warm rust ochre and vermilion highlights, about 16 flat colors. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. Input reference preserves base design or title layout, recreate in target style.
```

### pc98/props/hazards/log.png

評語：可交原圖；無特別內容重畫需求。仍需 Claude 做色盤、尺寸與錨點整理。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. ONE horizontal heavy round timber, side view, length 2.5 times diameter, left end shows circular endgrain growth rings, right extends horizontally with clear chunky bark grooves. Rolling left. Entire silhouette visible, landscape canvas. No motion arrows or motion effects. PC-98 pixel art matching viewed PC98 reference sprites and grounds: dark indigo purple contours, muted cool slate rock, warm rust ochre and vermilion highlights, about 16 flat colors. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. 
```

### pc98/ui/logo.png

評語：可交原圖；無特別內容重畫需求。仍需 Claude 做色盤、尺寸與錨點整理。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. Game title logo, exact Traditional Chinese text first line "虛擬", second line "闖天下", small bold English "VIRTUAL QUEST" below. Follow viewed original title layout: huge thick dynamic 1990s Chinese brush lettering, the character 闖 vermilion red, other characters cream, thick dark outline. Translate brush contours to crisp chunky pixel stair-steps. Horizontal overall canvas around 5:2; heavy legible strokes for small menu corner. No extra text. PC-98 pixel art matching viewed PC98 reference sprites and grounds: dark indigo purple contours, muted cool slate rock, warm rust ochre and vermilion highlights, about 16 flat colors. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. Input reference preserves base design or title layout, recreate in target style.
```

### msdos/props/barrel.png

評語：內容可交原圖；不滿意：有 EGA 外的近似過渡色，精確 EGA16 仍需後續整理。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. ONE upright gasoline barrel, bold flame hazard pictogram without words, warning band, front slightly above with elliptical top. Red body and yellow-black stripe except Game Boy must translate all colors to four green tones. Square canvas. MS-DOS EGA pixel art matching viewed MSDOS sprites and grounds: ONLY standard EGA16 colors #000000 #0000AA #00AA00 #00AAAA #AA0000 #AA00AA #AA5500 #AAAAAA #555555 #5555FF #55FF55 #55FFFF #FF5555 #FF55FF #FFFF55 #FFFFFF. Bold high contrast. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. 
```

### msdos/props/hazards/geyser-idle.png

評語：內容可交原圖；不滿意：有 EGA 外的近似過渡色，精確 EGA16 仍需後續整理。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. Single broad flat rocky ground vent, ellipse width 3.5 times height, dark hot interior, only two tiny angular wisps of steam. Oblique top/front view. Landscape canvas. MS-DOS EGA pixel art matching viewed MSDOS sprites and grounds: ONLY standard EGA16 colors #000000 #0000AA #00AA00 #00AAAA #AA0000 #AA00AA #AA5500 #AAAAAA #555555 #5555FF #55FF55 #55FFFF #FF5555 #FF55FF #FFFF55 #FFFFFF. Bold high contrast. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. 
```

### msdos/props/hazards/geyser-active.png

評語：已多次重畫並去掉明顯柔光；不滿意：active 與 idle 岩環同形程度不足，建議要求無縫切換時重畫底座。

提示詞（最後採用版）：

```text
Recolor the reference hard-edged sprite into msdos art. Keep its exact outlined silhouette, transparency and block pixels. MS-DOS EGA pixel art matching viewed MSDOS sprites and grounds: ONLY standard EGA16 colors #000000 #0000AA #00AA00 #00AAAA #AA0000 #AA00AA #AA5500 #AAAAAA #555555 #5555FF #55FF55 #55FFFF #FF5555 #FF55FF #FFFF55 #FFFFFF. Bold high contrast. Central vertical shape flat scarlet/red/yellow; rocks dark slate blue gray. STRICT FLAT INK, this is colored paper shapes, it does not emit light. NO glow, no halo, no bloom, no atmospheric light. Absolutely transparent background all the way to solid contour. Narrow bottom ellipse width to 60% so vertical shape height equals 2.5 times ellipse width. Do not add backdrop or shadow.
```

### msdos/props/hazards/spikes-idle.png

評語：內容可交原圖；不滿意：有 EGA 外的近似過渡色，精確 EGA16 仍需後續整理。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. Single wide shallow iron trap plate, oblique top/front view, one straight row of six dark empty holes, small corner rivets, slight chipped castle stone edge attached to plate only. Low flat silhouette, wide landscape canvas. MS-DOS EGA pixel art matching viewed MSDOS sprites and grounds: ONLY standard EGA16 colors #000000 #0000AA #00AA00 #00AAAA #AA0000 #AA00AA #AA5500 #AAAAAA #555555 #5555FF #55FF55 #55FFFF #FF5555 #FF55FF #FFFF55 #FFFFFF. Bold high contrast. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. 
```

### msdos/props/hazards/spikes-active.png

評語：已重畫加高尖刺，目視接近需求。底座與 idle 須對位，生成式重繪存在細部差異。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. Edit the reference idle trap plate into fully extended state. Preserve exact same six holes, plate silhouette, rivets and rock trim. Raise ONE sharp triangular iron spike from each hole. Each spike must project ABOVE THE BACK EDGE of plate by a clearly visible height equal to 20% of total plate width; these must be long upright pointed spikes, not little studs hidden in holes. Height 300 pixels for a 1500-pixel-wide plate. Entire spikes inside canvas, transparent margins. MS-DOS EGA pixel art matching viewed MSDOS sprites and grounds: ONLY standard EGA16 colors #000000 #0000AA #00AA00 #00AAAA #AA0000 #AA00AA #AA5500 #AAAAAA #555555 #5555FF #55FF55 #55FFFF #FF5555 #FF55FF #FFFF55 #FFFFFF. Bold high contrast. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. Input reference preserves base design or title layout, recreate in target style.
```

### msdos/props/hazards/log.png

評語：內容可交原圖；不滿意：有 EGA 外的近似過渡色，精確 EGA16 仍需後續整理。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. ONE horizontal heavy round timber, side view, length 2.5 times diameter, left end shows circular endgrain growth rings, right extends horizontally with clear chunky bark grooves. Rolling left. Entire silhouette visible, landscape canvas. No motion arrows or motion effects. MS-DOS EGA pixel art matching viewed MSDOS sprites and grounds: ONLY standard EGA16 colors #000000 #0000AA #00AA00 #00AAAA #AA0000 #AA00AA #AA5500 #AAAAAA #555555 #5555FF #55FF55 #55FFFF #FF5555 #FF55FF #FFFF55 #FFFFFF. Bold high contrast. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. 
```

### msdos/ui/logo.png

評語：內容可交原圖；不滿意：有 EGA 外的近似過渡色，精確 EGA16 仍需後續整理。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. Game title logo, exact Traditional Chinese text first line "虛擬", second line "闖天下", small bold English "VIRTUAL QUEST" below. Follow viewed original title layout: huge thick dynamic 1990s Chinese brush lettering, the character 闖 vermilion red, other characters cream, thick dark outline. Translate brush contours to crisp chunky pixel stair-steps. Horizontal overall canvas around 5:2; heavy legible strokes for small menu corner. No extra text. MS-DOS EGA pixel art matching viewed MSDOS sprites and grounds: ONLY standard EGA16 colors #000000 #0000AA #00AA00 #00AAAA #AA0000 #AA00AA #AA5500 #AAAAAA #555555 #5555FF #55FF55 #55FFFF #FF5555 #FF55FF #FFFF55 #FFFFFF. Bold high contrast. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. Input reference preserves base design or title layout, recreate in target style.
```

### gameboy/props/barrel.png

評語：內容可交原圖；不滿意：仍有生成過渡色，嚴格四色未達可保證狀態。若不能後續整理色盤則需重畫。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. ONE upright gasoline barrel, bold flame hazard pictogram without words, warning band, front slightly above with elliptical top. Red body and yellow-black stripe except Game Boy must translate all colors to four green tones. Square canvas. Original Game Boy pixel art matching viewed green references, ONLY four opaque colors #0F380F #306230 #8BAC0F #9BBC0F plus full transparency. No red, gray, orange or white. Coarse 64px-like sprite logic. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. 
```

### gameboy/props/hazards/geyser-idle.png

評語：內容可交原圖；不滿意：仍有生成過渡色，嚴格四色未達可保證狀態。若不能後續整理色盤則需重畫。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. Single broad flat rocky ground vent, ellipse width 3.5 times height, dark hot interior, only two tiny angular wisps of steam. Oblique top/front view. Landscape canvas. Original Game Boy pixel art matching viewed green references, ONLY four opaque colors #0F380F #306230 #8BAC0F #9BBC0F plus full transparency. No red, gray, orange or white. Coarse 64px-like sprite logic. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. 
```

### gameboy/props/hazards/geyser-active.png

評語：不滿意：柱高與洞口寬比例不足，建議重畫。四色精確值亦需檢查。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. Single broad flat rocky ground vent at bottom, ellipse width 3.5 times its height. From its center erupts ONE tall straight vertical magma column, height 2.5 times vent width, several lava blobs spray at top. Whole image tall portrait, entire column and vent visible, margins all around. No floating platform. Original Game Boy pixel art matching viewed green references, ONLY four opaque colors #0F380F #306230 #8BAC0F #9BBC0F plus full transparency. No red, gray, orange or white. Coarse 64px-like sprite logic. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. Input reference preserves base design or title layout, recreate in target style.
```

### gameboy/props/hazards/spikes-idle.png

評語：內容可交原圖；不滿意：仍有生成過渡色，嚴格四色未達可保證狀態。若不能後續整理色盤則需重畫。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. Single wide shallow iron trap plate, oblique top/front view, one straight row of six dark empty holes, small corner rivets, slight chipped castle stone edge attached to plate only. Low flat silhouette, wide landscape canvas. Original Game Boy pixel art matching viewed green references, ONLY four opaque colors #0F380F #306230 #8BAC0F #9BBC0F plus full transparency. No red, gray, orange or white. Coarse 64px-like sprite logic. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. 
```

### gameboy/props/hazards/spikes-active.png

評語：已重畫加高尖刺，目視接近需求。底座與 idle 須對位，生成式重繪存在細部差異。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. Edit the reference idle trap plate into fully extended state. Preserve exact same six holes, plate silhouette, rivets and rock trim. Raise ONE sharp triangular iron spike from each hole. Each spike must project ABOVE THE BACK EDGE of plate by a clearly visible height equal to 20% of total plate width; these must be long upright pointed spikes, not little studs hidden in holes. Height 300 pixels for a 1500-pixel-wide plate. Entire spikes inside canvas, transparent margins. Original Game Boy pixel art matching viewed green references, ONLY four opaque colors #0F380F #306230 #8BAC0F #9BBC0F plus full transparency. No red, gray, orange or white. Coarse 64px-like sprite logic. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. Input reference preserves base design or title layout, recreate in target style.
```

### gameboy/props/hazards/log.png

評語：內容可交原圖；不滿意：仍有生成過渡色，嚴格四色未達可保證狀態。若不能後續整理色盤則需重畫。

提示詞（最後採用版）：

```text
Use case: stylized-concept. Asset: ONE isolated beat-em-up game sprite for Virtual Quest. ONE horizontal heavy round timber, side view, length 2.5 times diameter, left end shows circular endgrain growth rings, right extends horizontally with clear chunky bark grooves. Rolling left. Entire silhouette visible, landscape canvas. No motion arrows or motion effects. Original Game Boy pixel art matching viewed green references, ONLY four opaque colors #0F380F #306230 #8BAC0F #9BBC0F plus full transparency. No red, gray, orange or white. Coarse 64px-like sprite logic. Clear BIG square pixel grid, hard stepped contours, limited flat color clusters, sparse ordered dithering, absolutely NO gradients, NO bloom, NO glow halos, NO antialiasing, NO soft or translucent pixels. Upper-left light. Actual fully transparent empty background, complete silhouette with margin on all sides. No ground plane or cast shadow. No watermark. No text except logo. 
```



## 本批 PNG 畫布尺寸

- pc98\props\barrel.png: 1254 x 1254; RGBA PNG
- pc98\props\hazards\geyser-active.png: 1060 x 1484; RGBA PNG
- pc98\props\hazards\geyser-idle.png: 1855 x 848; RGBA PNG
- pc98\props\hazards\log.png: 1774 x 887; RGBA PNG
- pc98\props\hazards\spikes-active.png: 1774 x 887; RGBA PNG
- pc98\props\hazards\spikes-idle.png: 1774 x 887; RGBA PNG
- pc98\ui\logo.png: 1983 x 793; RGBA PNG
- msdos\props\barrel.png: 1254 x 1254; RGBA PNG
- msdos\props\hazards\geyser-active.png: 1060 x 1484; RGBA PNG
- msdos\props\hazards\geyser-idle.png: 1774 x 887; RGBA PNG
- msdos\props\hazards\log.png: 1774 x 887; RGBA PNG
- msdos\props\hazards\spikes-active.png: 1774 x 887; RGBA PNG
- msdos\props\hazards\spikes-idle.png: 1774 x 887; RGBA PNG
- msdos\ui\logo.png: 1983 x 793; RGBA PNG
- gameboy\props\barrel.png: 1254 x 1254; RGBA PNG
- gameboy\props\hazards\geyser-active.png: 1060 x 1484; RGBA PNG
- gameboy\props\hazards\geyser-idle.png: 1659 x 948; RGBA PNG
- gameboy\props\hazards\log.png: 1774 x 887; RGBA PNG
- gameboy\props\hazards\spikes-active.png: 1774 x 887; RGBA PNG
- gameboy\props\hazards\spikes-idle.png: 1774 x 887; RGBA PNG
