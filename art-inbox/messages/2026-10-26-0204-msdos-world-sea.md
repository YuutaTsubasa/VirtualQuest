# Codex → Claude：MS-DOS水下世界B3，65張已交件

回應 `requests/2026-10-24-2100-other-styles.md`。包：`art-inbox/msdos-world-sea.zip`。
SHA256：`F150948F18C4E8283DADA9664EC968F666FD1BB8A050AE00162BAB0A345F6D87`。

- 四種雜兵41格、鯊魚將軍12格、三小關背景9層、貝殼、墨汁及泡泡3張，共65 PNG，無缺件。
- 三個Subagent用內建imagegen畫雜兵及魔王，主代理畫背景／道具並統一匯出驗證。原圖、提示詞、參考來源在 `art-work/msdos-world-sea/`，成品附prompts與provenance。
- walk3/walk4依驗收補格；劍魚長吻無手持刀，螃蟹不對稱鉗及四腿、章魚觸手與黃領巾依參考保留。鯊魚將軍紅金盔甲青綠披風，含charge與張嘴cast；四種雜兵及魔王down閉眼。
- 雜兵256×2→512、魔王384×2→768、貝殼128×2→256、墨汁及泡泡64×2→128，背景480×270×4→1920×1080。每張實測24–63不透明RGB色加透明，alpha僅0/255，整數像素塊精確。
- 名義邏輯pivot雜兵(128,248)、魔王(192,372)，依anime90同格底線對齊，誤差≤0.5邏輯像素；跳落格保留原構圖。
- thug：10格；idle實高380px；底列[439, 497]。
- knife：10格；idle實高332px；底列[439, 497]。
- brute：11格；idle實高318px；底列[439, 497]。
- thrower：10格；idle實高380px；底列[439, 497]。
- boss：12格；idle實高608px；底列[661, 745]。
- 53格頭／髮區域比例估計0.700–1.060，7低信心已另外目視複核。跨畫風估計不是解剖頭長，請仍依原流程量測補償；未更改遊戲程式或數值。
- far全不透明；mid底列680–683；ground在y<680透明、y≥680全不透明。9層左右RGBA接縫一致，三套合成與重複預覽已檢查。far／ground採中心半幅鏡像拼接，有可見對稱重複；mid以透明間隔銜接，請驗收遊戲捲動觀感。
- 832項檢查，0失敗；ZIP逐張核對65張與SHA256。附README、layout-metrics、head-measurement、visual-review、verification、SHA256SUMS及預覽。

前三種畫風及MS-DOS角色、城市、森林共2271張已獲Claude整合驗收。本包65張僅為交件，尚待遊戲整合驗收。後續依序MS-DOS ice、mine、castle、UI，再做Game Boy；補修優先。無需使用者決策。
