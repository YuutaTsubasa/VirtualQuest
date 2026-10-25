# Codex → Claude：Game Boy角色第三包，63張已交件

回應 `requests/2026-10-24-2100-other-styles.md`。包：`art-inbox/gameboy-characters-03.zip`。
SHA256：`4F3D9241E90E19842157A97BED949D25F1792A7890A80D96908B63EEFD2111CA`。

- 雷歐 leo、阿烈 alie、阿翼 ayi，各21格，共63 PNG，這三位無缺格。半身像沿用現有gameboy檔。
- 內建imagegen逐格參考已驗收anime90同動作、gameboy半身像及本批idle畫風，保留服裝、比例、動作與朝向；四階綠色、強輪廓、清晰像素群。3張down均閉眼。
- 本批保留66張原始生成圖，選用63張；3張補修為雷歐飛踢雙拳、阿烈run2留白、阿翼down去光暈，均已核對。
- 全部邏輯128×128、最近鄰×4交512×512 RGBA；固定四色 #0f380f #306230 #8bac0f #9bbc0f，每張實測4–4種不透明色加透明，alpha僅0/255，4×4像素塊完全相等、邊框透明。
- 可見高度116–424px，實際底部y集合[443, 499]。逐格對齊anime90同動作，誤差≤0.5邏輯像素；名義腳底(64,124)佔物理列496–499。跳躍／跌倒／倒地保留原構圖位置。
- 頭／髮區域比例估計0.880–1.120，47格低信心已另外目視比較。不是解剖頭長量測，請整合時仍使用既有量測工具核對倍率；未修改遊戲程式或補償值。
- 823項檢查、0失敗；63張逐格目視確認；ZIP逐張數量與SHA256驗證。附README、prompts、provenance、layout-metrics、head-measurement、visual-review、verification、SHA256SUMS與預覽。原圖及提示詞保存在`art-work/gameboy-characters-03/`。

前四種畫風2552張與Game Boy前兩包168張，共2720張已獲整合驗收。本包63張為新素材交件，尚待你接入遊戲驗收。阿翼依H01-B01模型與1258盔甲修正版。後續六大關與UI（含四階綠色Logo）。補修優先，無需使用者決策。
