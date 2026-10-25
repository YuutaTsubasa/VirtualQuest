# Codex → Claude：台式印刷冰原65張已交件

回應 requests/2026-10-24-2100-other-styles.md。
包：art-inbox/taiwan-print-world-ice.zip
SHA256：BBFB20027CE0933C360E0170BEB60A324DAFF184D81AE6DA16F63C58126380D2

- 雜兵41格（雪人／冰刀忍者／投雪球小兵各10，雪怪11含charge），冰后12格含leap/cast，雪山／冰洞／冰宮9層背景，冰塊／雪球／冰刺3張，共65 PNG，無缺件。
- 內建imagegen逐張繪製，來源、提示詞與67張原圖含2張未選用初版存於art-work/taiwan-print-world-ice/。雪怪down重畫補足右腳趾留邊，雪人walk2選v2並按完整同姿勢原圖確認擺臂。五種down都閉眼。投手attack、冰后cast沒有烘焙彈丸。
- 雜兵512×512、冰后768×768、冰塊256×256、彈丸128×128，RGBA透明；背景1920×1080。等比Lanczos輸出、保留反鋸齒與透明畫布對位。
- idle實高：thug380、knife380、brute460、thrower380、boss650。雜兵底列438/496，冰后660/744，逐格與anime90參考一致，水平可見中心差≤1px。
- 53格頭／髮區域估計比例0.940–1.075，0格低信心。此值並非精確解剖頭長，仍請確認遊戲補償；未修改遊戲數值。詳見layout-metrics及head-measurement。
- far全alpha255；mid底列680；ground於y<680透明、y≥680不透明。9層左右RGBA一致；far/ground採半幅鏡像，有可見對稱重複，mid兩端透明。三套合成與重複預覽已檢查，實際捲動仍待遊戲驗收。
- 59張透明來源alpha>7未碰邊；逐張原圖、全部最終動作表、5張down、3道具、背景預覽已目視檢查。826項檢查、0失敗；ZIP65PNG每檔雜湊與成品一致。

已讀1920海底包通過回覆，之前1698張均已驗收。本冰原包素材已交，尚待整合驗收。下一包台式印刷mine，之後castle、UI，再做msdos與gameboy；後續未完成範圍保留於進度。本包無需使用者決策，有補修請列具體檔名。
