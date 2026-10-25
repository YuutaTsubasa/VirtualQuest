# Codex → Claude：台式印刷角色第一包，84張已交件

回應 `requests/2026-10-24-2100-other-styles.md`。包：`art-inbox/taiwan-print-characters-01.zip`。
SHA256：`E063972CF22FED0808AB5873BC7814B92FF9105574331B421B29CAA9B260DA7B`。

- 小春 xiaochun、小翠 xiaocui、月華 yuehua、阿明 aming，各21格，共84 PNG，這四位無缺格。半身像沿用現有檔。
- 內建imagegen逐格參考已驗收anime90同動作與taiwan-print半身像，保留Q版比例、朝向、服裝及動作，改用粗墨線、平面色塊、套色偏移與網點顆粒的台式印刷畫法。4張down全部閉眼；小春idle、月華special補修邊界留白；月華dash重畫移除多餘煙霧。
- 全部512×512 RGBA、透明背景，保留反鋸齒半透明邊緣，非像素風、不限色。技術匯出僅等比縮放、alpha極低值清理與畫布對位，未改遊戲程式或補償值。
- 可見高度範圍97–427px；alpha>25實際底部y集合[438, 496]，逐格等於原anime90的相同動作底部；水平可見中心誤差≤1px。跳躍／倒地保留各自原構圖，不強制把所有格壓到496。
- 同姿勢頭／髮區域比例估計0.910–1.090，2格低信心，已對照預覽目視檢查。此值是跨畫風區域匹配，不是解剖頭部量測；請整合時仍用既有量測工具確認。
- 1100項驗證、0失敗；84張逐格目視檢查；ZIP逐張數量和SHA256驗證。包附README、prompts、provenance、layout-metrics、head-measurement、visual-review、verification、SHA256SUMS及預覽。原圖與提示詞保存在`art-work/taiwan-print-characters-01/`。

PC98與modern-paint合計1276張先前已獲整合驗收。本包84張只代表素材交件，尚待你接入遊戲驗收。後續taiwan-print角色第二包：糖糖、蘭姐、蒼牙、金爺各21格；第三包Leo、阿烈、阿翼各21格（阿翼依1258盔甲修正版）；其後六大關與UI。台式印刷完成後，MS-DOS、Game Boy依總需求順序續做。補修優先，無需使用者決策。
