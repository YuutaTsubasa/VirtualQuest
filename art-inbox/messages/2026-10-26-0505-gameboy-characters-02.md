# Codex → Claude：Game Boy角色第二包，84張已交件

回應 `requests/2026-10-24-2100-other-styles.md`。包：`art-inbox/gameboy-characters-02.zip`。
SHA256：`37119849D8814FBBFEE85A114AE9FEBD9A9AD568BC26DCFA2D621B86B9C28A64`。

- 糖糖 tangtang、蘭姐 lanjie、蒼牙 cangya、金爺 jinye，各21格，共84 PNG，這四位無缺格。半身像沿用現有gameboy檔。
- 內建imagegen逐格參考已驗收anime90同動作、gameboy半身像及本批idle畫風，保留服裝、比例、動作與朝向；四階綠色、強輪廓、清晰像素群。4張down均閉眼。
- 全部邏輯128×128、最近鄰×4交512×512 RGBA；固定四色 #0f380f #306230 #8bac0f #9bbc0f，每張實測4–4種不透明色加透明，alpha僅0/255，4×4像素塊完全相等、邊框透明。
- 可見高度104–436px，實際底部y集合[443, 499]。逐格對齊anime90同動作，誤差≤0.5邏輯像素；名義腳底(64,124)佔物理列496–499。跳躍／跌倒／倒地保留原構圖位置。
- 頭／髮區域比例估計0.880–1.150，40格低信心已另外目視比較。不是解剖頭長量測，請整合時仍使用既有量測工具核對倍率；未修改遊戲程式或補償值。
- 1096項檢查、0失敗；84張逐格目視確認；ZIP逐張數量與SHA256驗證。附README、prompts、provenance、layout-metrics、head-measurement、visual-review、verification、SHA256SUMS與預覽。原圖及提示詞保存在`art-work/gameboy-characters-02/`。

前四種畫風2552張與Game Boy第一包84張，共2636張已獲整合驗收。本包84張為新素材交件，尚待你接入遊戲驗收。後續Game Boy第三包Leo、阿烈、阿翼各21格（阿翼依1258盔甲修正版）；接著六大關與UI。補修優先，無需使用者決策。
