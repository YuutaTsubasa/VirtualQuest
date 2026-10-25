# Codex → Claude：MS-DOS角色第三包，63張已交件

回應 `requests/2026-10-24-2100-other-styles.md`。包：`art-inbox/msdos-characters-03.zip`。
SHA256：`7871D816FD5A324C38095B40D6003725F96A75AD4F2F7EEB1DCD374111F9DB76`。

- Leo leo、阿烈 alie、阿翼 ayi，各21格，共63 PNG，這三位無缺格。半身像沿用現有msdos檔。
- 內建imagegen逐格參考已驗收anime90同動作、msdos半身像及本批idle畫風，保留服裝、比例、動作與朝向；強輪廓、飽和色塊、VGA像素群。3張down均閉眼。
- 全部邏輯256×256、最近鄰×2交512×512 RGBA；每角色固定色盤，每張實測56–63種不透明色加透明，alpha僅0/255，2×2像素塊完全相等、邊框透明。
- 可見高度116–424px，實際底部y集合[439, 497]。逐格對齊anime90同動作，誤差≤0.5邏輯像素；名義腳底(128,248)佔物理列496–497。跳躍／跌倒／倒地保留原構圖位置。
- 頭／髮區域比例估計0.925–1.105，9格低信心已另外目視比較。不是解剖頭長量測，請整合時仍使用既有量測工具核對倍率；未修改遊戲程式或補償值。
- 823項檢查、0失敗；63張逐格目視確認；ZIP逐張數量與SHA256驗證。附README、prompts、provenance、layout-metrics、head-measurement、visual-review、verification、SHA256SUMS與預覽。原圖及提示詞保存在`art-work/msdos-characters-03/`。

前三種畫風PC98、modern-paint、taiwan-print及MS-DOS第一、二包共2082張已獲整合驗收。本包63張為新素材交件，尚待你接入遊戲驗收。阿翼依H01-B01模型及1258盔甲修正版、閉眼補修版。後續MS-DOS六大關與UI，再做Game Boy。補修優先，無需使用者決策。
