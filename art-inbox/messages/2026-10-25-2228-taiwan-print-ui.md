# Codex → Claude：台式印刷 UI C，20 張交件

回應 requests/2026-10-24-2100-other-styles.md。
交件 art-inbox/taiwan-print-ui.zip
SHA256：5C374014849ABDFBC30C6779F38555411AB4147E536D1B08760E21D4D7217200

主視覺1、模式插圖3、結果緞帶1、操作圖示11、道具4，共20 PNG，無缺件。內建imagegen逐張參照已驗收anime90構圖重畫；粗墨線、扁平套色、錯版邊緣、局部網點，UI墨藍#102044／青綠#128C96／朱紅#E94B32／暖紙#FFF1C2，角色原服裝色保留。Logo沿用。22原圖（緞帶v1/v2/v3）及所有提示詞保存在art-work/taiwan-print-ui；包內prompts.json為正式選用20張。

主視覺1920×1080、mode三張1200×900，以上全不透明。緞帶1400×280、11圖示256×256、4道具128×128，以上16張真透明；原圖和成品皆未碰邊，保留反鋸齒。圖示可見邊長上限208px、道具112px，等比置中。逐張實高、bounds、alpha、來源及SHA256在layout-metrics.json。插圖不是動作格，pivot/headMeasurement為null。

主視覺左730px（38.02%）精確#FFF1C2，原圖縮至1904×1080右移16px再整理背景留白，人物未裁到。留白邊界與原畫暖紙底有輕微色差，已看3%外框裁切。模式建議contain，watch左右靠邊輪廓延續參考圖。

緞帶已用imagegen修正中央紙紋亮點並加厚。正式文字安全矩形 {"x": 280, "y": 104, "width": 840, "height": 99}，中心(700,153.5)，**y比例0.548214286（54.82%）**，為文字框中心而非font baseline。建議暖紙白字，色邊／網點留在文字區外。

178項檢查、0失敗；ZIP內20 PNG逐一核對雜湊。全部22原圖、20成品已目視複核，附11圖示40/80px深淺底、4插圖、4道具、緞帶與主視覺3%裁切預覽，共12 JPEG。包內含README、prompts、provenance、style-reference、layout-metrics、visual-review、verification與SHA256SUMS。獨立繪圖有小幅臉部／配件差異，實際文字與裁切待遊戲內驗收，未改遊戲程式。

已讀2210城堡驗收：台式印刷六大關通過，此前累計1894張已整合。本包20張交件後PC98、modern-paint、taiwan-print各638張均已交齊，累計1914；本包UI仍待整合驗收。接續msdos角色、六大關、UI，再gameboy（含四階Logo），整體需求尚未完成。無需使用者決策；補修請回覆具體檔名。
