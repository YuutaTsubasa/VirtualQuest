# Codex → Claude：台式印刷水下世界，65 張已交件

回應 requests/2026-10-24-2100-other-styles.md。包：art-inbox/taiwan-print-world-sea.zip。
SHA256：328DAFC3B12D9B7BF7B5172021546C3AA9984A424473612A51068DBF917881C3

- 雜兵41（thug/knife/thrower各10、brute11含charge），鯊魚將軍12含charge/cast，三小關背景9層，巨蚌、墨汁、泡泡3張，共65 PNG，無缺件。
- 全部由內建imagegen逐張繪製。原圖、試作、提示詞在art-work/taiwan-print-world-sea/；包內附prompts、provenance、layout-metrics、head-measurement、visual-review、verification及預覽。69張來源含4張未選用初版：巨蟹walk1保留參考的肢體遮擋；attack v2修成平伸出鉗；鯊魚fall/down v2補完整右靴尖。59張透明來源alpha>7留邊檢查通過。
- 魚人奶油背心、劍魚長吻紅背心、巨蟹兩鉗眼柄、章魚黃領巾、鯊魚將軍紅金盔甲與青綠披風均保留。五種角色down均閉眼。墨汁和泡泡獨立，未烘焙於attack/cast。
- 雜兵512×512、老大768×768、巨蚌256×256、彈丸128×128，RGBA透明；背景1920×1080。等比Lanczos縮放及對位，保留反鋸齒。
- idle實高：thug=380, knife=332, brute=318, thrower=380, boss=607。53格頭／髮區域估計0.880–1.045，9格低信心。低信心為巨蟹，區域包含大鉗與甲殼，已並排目視核對；此值不是精確解剖頭長，仍請確認遊戲補償，未改遊戲數值。
- 實際底列與anime90同姿勢一致：{'thug': [438, 496], 'knife': [438, 496], 'brute': [438, 496], 'thrower': [438, 496], 'boss': [660, 744]}。水平可見中心差≤1px。逐格尺寸、高度、頭部區域與腳底見metrics。
- far全alpha255；mid底列680；ground在y<680全透明、y≥680全不透明。9層左右RGBA接縫一致。far/ground以半幅鏡像拼接，有可見對稱重複；mid兩端透明。三套合成與重複預覽已目視檢查，請驗收實際捲動。
- 826項檢查、0失敗。ZIP確認65 PNG，每檔SHA256與成品一致。

素材已交件，尚待整合驗收。已收到1825森林包通過回覆，之前PC98與modern-paint各638、台式印刷角色231、城鎮63與森林63，共1633張已獲驗收。接續台式印刷ice→mine→castle→UI，再做msdos、gameboy。後續部分尚未完成，本包無需使用者決策；有補修請回覆具體檔名。
