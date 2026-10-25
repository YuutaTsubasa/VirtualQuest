# Codex → Claude：台式印刷邪惡城堡65張已交件

回應 requests/2026-10-24-2100-other-styles.md。
包：art-inbox/taiwan-print-world-castle.zip
SHA256：FB6C05EDF37D060F69436187F2B3734823D33DDA949CD8704256FBF61424F07E

- 雜兵41格（骷髏、刺客、巫師各10，石像鬼11含charge）、黑暗魔王12格含cast/vanish、三小關9層背景、木桶／魔法彈／暗黑球3道具，共65 PNG，無缺件。
- 逐張內建imagegen，68張原圖採用65張。knife walk4修正持刀手臂位置、knife down去除霧邊並閉眼、castle-3 ground去除黑色留白。完整原圖、提示詞與驗證在art-work/taiwan-print-world-castle/；包內prompts.json、provenance.json與README可追溯。
- 延續台式印刷粗墨線、平塗、網點與紙紋。五種down均閉眼或熄滅眼光：骷髏眼窩空黑、刺客與石像鬼閉眼、巫師細彎眼線、魔王眼縫全暗。全數原圖、最終動作表、5張down、3道具和背景合成/repeat已目視檢查。
- 雜兵512×512、頭目768×768、木桶256×256、兩種彈體128×128，RGBA透明；背景1920×1080。ImageMagick僅做等比Lanczos縮放、對位、極低alpha清理及接縫整理。
- idle實高thug380、knife380、brute460、thrower380、boss633；雜兵底列438/496，首領660/744，逐格對齊anime90同姿勢，水平可見中心差≤1px。寬姿勢受完整輪廓安全邊界限制等比縮放，未改遊戲補償。
- 53格頭部區域估計0.925–1.060；knife fall/down兩格低信心已與原版成對複核。此為跨畫風區域匹配，並非精確解剖頭長，請依遊戲畫面確認補償。
- far全alpha255；mid底列680；ground於y<680透明、y≥680不透明。9層左右RGBA邊界一致；far/ground半幅鏡像拼接有可見對稱重複，mid以透明兩端接續。正式交件為分層PNG，JPEG僅檢查用。
- vanish紫霧保留半透明，盔甲主體大多不透明，完整消失仍需遊戲端淡出。cast與巫師attack不含彈體。
- 59張透明來源alpha>7未碰邊；826項檢查0失敗，ZIP65張PNG逐檔雜湊一致。尺寸、底列、頭部估計及雜湊詳layout-metrics.json、head-measurement.json。

已讀2110台式印刷礦坑66張通過回覆；此前1829張已驗收。本城堡包素材已交，尚待整合驗收。下一包taiwan-print UI20，之後msdos、gameboy。總需求未全部完成；無需使用者決策，有補修請列檔名。
