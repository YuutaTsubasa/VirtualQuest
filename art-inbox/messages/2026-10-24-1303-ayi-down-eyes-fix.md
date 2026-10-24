# 阿翼倒地改成閉眼：單張補丁

回應本對話使用者新回饋「倒地好像臉的部份眼睛是張開的有點怪」，屬於 `requests/2026-10-24-0157-anime90-batch1-characters.md` 的使用者驗收補修。

已交 `art-inbox/anime90-ayi-down-eyes-fix.zip`，共 **1 張 PNG**。請在 `anime90-ayi-model-fix.zip` 19 格修正版之後，**只覆蓋 `characters/ayi/sprites/down.png`**，其餘 18 格沿用盔甲修正版。

改為閉合眼瞼、無藍色虹膜或眼白，嘴角放鬆，保留倒地姿勢與 H01-B01 盔甲、鞋跟設計。正式 512×512 RGBA，可見範圍 (76,381)–(435,496)，360×116，邏輯 pivot (256,496)，與上一版相同。

8 項檢查通過、ZIP 檔案數及內容雜湊核對通過；放大目視已確認表情。輪廓 IoU 98.6248%，輪廓差異均在 1 像素內；生成改圖仍有細微線條差異，非完全像素鎖定。來源、提示詞及驗證保存在 `art-work/anime90-ayi-down-eyes-fix/`，比較圖在包內 `previews/down-compare.jpg`。

ZIP SHA256：`7546408A1240C7B31B020700859DA149DD0EE9E0C5A0FAC800D2CA16B3C24F33`

本次補修無未完成素材；尚待你接入遊戲並驗收，請回覆確認載入的是閉眼版本。
