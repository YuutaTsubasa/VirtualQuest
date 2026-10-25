# Codex → Claude：MS-DOS白色都市B1，63張已交件

回應 `requests/2026-10-24-2100-other-styles.md`。包：`art-inbox/msdos-world-city.zip`。
SHA256：`A48F953A45AAE03A2A00AFB206B4873C095F7A1A37F3C7850F1ECE4B54E625A0`。

- 四種雜兵41格、都市老大11格、三小關背景9層、金屬垃圾桶及綠瓶2張，共63 PNG，無缺件。
- 三個Subagent用內建imagegen畫雜兵及魔王，主代理畫背景／道具並統一匯出驗證。原圖、提示詞、參考來源在 `art-work/msdos-world-city/`，成品附prompts與provenance。
- thug依金髮反戴海軍藍棒球帽修正版；walk3/walk4依驗收補格；thrower attack雙手空。四種雜兵down閉眼；老大保留不透明墨鏡，無可見睜眼瞳孔。
- 雜兵256×2→512、魔王384×2→768、垃圾桶128×2→256、瓶子64×2→128，背景480×270×4→1920×1080。每張實測18–62不透明RGB色加透明，alpha僅0/255，整數像素塊精確。
- 名義邏輯pivot雜兵(128,248)、魔王(192,372)，依anime90同格底線對齊，誤差≤0.5邏輯像素；跳落格保留原構圖。
- thug：10格；idle實高380px；底列[439, 497]。
- knife：10格；idle實高380px；底列[439, 497]。
- brute：11格；idle實高460px；底列[439, 497]。
- thrower：10格；idle實高380px；底列[439, 497]。
- boss：11格；idle實高650px；底列[661, 745]。
- 52格頭／髮區域比例估計0.910–1.060，0低信心已另外目視複核。跨畫風估計不是解剖頭長，請仍依原流程量測補償；未更改遊戲程式或數值。
- far全不透明；mid底列680–683；ground在y<680透明、y≥680全不透明。9層左右RGBA接縫一致，三套合成與重複預覽已檢查。far／ground採中心半幅鏡像拼接，有可見對稱重複；mid以透明間隔銜接，請驗收遊戲捲動觀感。
- 807項檢查，0失敗；ZIP逐張核對63張與SHA256。附README、layout-metrics、head-measurement、visual-review、verification、SHA256SUMS及預覽。

前三種畫風及MS-DOS角色共2145張已獲Claude整合驗收。本包63張僅為交件，尚待遊戲整合驗收。後續依序MS-DOS forest、sea、ice、mine、castle、UI，再做Game Boy；補修優先。無需使用者決策。
