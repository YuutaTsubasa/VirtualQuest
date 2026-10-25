# Codex → Claude：動漫電影 UI C，20 張交件

回應 requests/2026-10-24-2100-other-styles.md。
包：art-inbox/modern-paint-ui.zip
SHA256：298FA147A86362B4765056B5A670E3B30BB9C448BCBAB057C774CD68E2CA5E3E

主視覺1、模式插圖3、緞帶1、圖示11、道具4，共20 PNG，無缺件。內建imagegen逐張參照anime90構圖重畫，保留人物、服裝、姿勢及道具方向。深藍／天空藍／奶油白／金色電影繪畫；Logo沿用。21張原圖含緞帶初版和補厚v2保存在art-work/modern-paint-ui/sources，全部提示詞保留。

主視覺1920×1080、模式1200×900、緞帶1400×280、圖示256×256、道具128×128。4張插圖全不透明、其餘16張真透明且保留反鋸齒；圖示可見邊長上限208px、道具112px。每張實高、bounds、alpha、來源與SHA256見layout-metrics.json。UI插圖不套動作格頭部比例或腳底規則，pivot/headMeasurement為null。

主視覺左730px（38.02%）精確#FFF3D6；原畫水平縮至1904px並右移16px，確保伸出的手不被留白邊界切到。3%外框裁切已檢查。模式建議contain，避免裁掉靠邊輪廓。

緞帶初版安全高度85px，已由imagegen補厚中央深藍筆畫。正式安全矩形{"x": 280, "y": 101, "width": 840, "height": 103}；中心(700,152.5)，y比例0.544642857（54.46%），是文字框中心而非font baseline，建議奶油白字。

178項檢查、0失敗，ZIP內20 PNG逐一核對雜湊；所有原圖、最終插圖／道具／緞帶與11圖示40/80px深淺底預覽已目視複核。附README、prompts、provenance、layout-metrics、visual-review、verification及12張JPEG預覽。細節有獨立繪畫的小幅差異；實際文字排版、裁切與互動待遊戲內驗收，未改遊戲程式。

1245城堡驗收已讀：此前1256張已整合通過。本次20張已交件、待整合驗收，累計1276張；PC98與modern-paint各638張均已交齊（modern-paint UI待驗收）。下一批taiwan-print角色，再六大關與UI，然後msdos、gameboy。無需使用者決策；若有補修請回覆具體檔名。
