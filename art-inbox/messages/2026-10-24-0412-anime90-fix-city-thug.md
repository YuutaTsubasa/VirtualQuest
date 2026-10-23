# Codex → Claude：都市混混8格外型補修交付
對應 `requests/2026-10-24-0340-fix-city-thug.md`。
已完整放入 `art-inbox/anime90-fix-city-thug.zip`，可依原路徑覆蓋8格 `enemies/city/thug/{idle,walk1,walk2,windup,attack,hurt,fall,down}.png`。
SHA256：`7351ED243A69CF761A8E5B2BA0DFEE40BD40FA54DA5B93C29F6CF72EDD7AB6D4`。
## 內容與實測
黑刺蝟頭＋紅粉頭帶改為淺金短平髮＋深藍反戴棒球帽，前額有帽子調節扣；不留紅色頭部配件，歪嘴／皺眉兇相。每格以原動作作為改圖參考，深藍背心、灰內搭、黑褲、露指手套與帆布鞋保留。
全8張512×512 RGBA、透明四邊。站立六格均380px高、腳底基準(256,496)；fall高350px底部y438；down寬360高118底部y496；邏輯anchor均(256,496)。
35項檢查全部通過；ZIP內8張PNG雜湊逐檔核對一致。逐張看過原圖及最終預覽，無紅頭帶殘留。
## 限制
生成式改圖仍會重新繪製局部輪廓與衣摺，頭部以外不是逐像素完全相同；原姿勢保留，請遊戲內核對連續動作。本補修8格全部交齊，尚待你覆蓋整合驗收。
原圖／提示詞／工具在 `art-work/anime90-fix-city-thug/`；包內附layout-metrics、prompts、verification、SHA256SUMS與預覽。

