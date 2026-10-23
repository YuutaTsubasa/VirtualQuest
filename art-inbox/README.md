# 美術收件匣（給 ChatGPT／Codex）

Claude Code 在背景監看這個資料夾。**放進新檔案、而且兩分鐘內沒再變動**，Claude 就會醒來處理：接進遊戲、檢查畫面、在本機 commit。

## 怎麼交件

1. 每種風格做好後，把整包放到這裡，二選一：
   - 壓縮檔：`art-inbox/<風格>-assets.zip`
   - 資料夾：`art-inbox/<風格>/`（**先在別處做好再整個搬進來**，免得做到一半就被讀走）
2. 放好後在 `art-inbox/messages/` 寫一則留言，檔名用 `時間-風格.md`，例如 `2026-10-23-0600-pc98.md`。內容寫：
   - 這包是什麼、有幾張、哪些還沒做
   - 跟 90 年代版不一樣的地方（例如尺寸、錨點、調色盤限制）
   - 有沒有要 Claude 特別注意或決定的事

只寫留言、不放檔案也可以（例如「pc98 要晚點，先做 gameboy」），Claude 一樣會讀。

> 你原本的輸出資料夾（`~/.codex/visualizations/.../volleyball/`）裡新出現的 `.zip` 也會被看到，但放在這裡最確定。

## 風格 ID（跟 TaiwanMahjong 一樣）

| ID | 名稱 |
| --- | --- |
| `anime90` | 90 年代（已完成，`docs/art/anime90-v1/`） |
| `pc98` | PC-98 |
| `msdos` | MS-DOS |
| `taiwan-print` | 台式印刷 |
| `gameboy` | Game Boy |
| `modern-paint` | 動漫電影 |

## 內容規格：跟 90 年代圖素包相同

照 `anime90-full-v1-assets.zip` 的結構就好，Claude 已經會處理那個格式：

```
characters/<角色>/portraits/{normal,happy,surprised,sad,win}.png   512×512 透明
characters/<角色>/sprites/{idle,run1,run2,jump,spike,dive,win,lose}.png   512×512 透明、面向右、腳底在 (256, 496)
environment/court.png   1920×1080（下方約 36% 是沙地，不要畫網）
environment/net.png     側視球網、透明
environment/ball.png    256×256 透明
effects/{hit-spark,sand-puff,score-burst,spike-trail}.png
ui/logo.png、ui/score-panel.png、ui/result-banner.png、ui/icons/*.png
manifest.json、README、prompts（有的話）
```

- 角色資料夾名稱用正式 ID 或圖素包的 ID 都可以：`xiaocui` `yuehua` `aming` `tangtang` `xiaochun` `lanjie` `cangya`(01-wolf) `jinye`(02-gambler) `leo`(03-prince) `alie`(04-adventurer) `ayi`。
- 像素風格（PC-98、MS-DOS、Game Boy）請用**整數倍放大**交 512，Claude 會用最近鄰縮放，不會糊掉。
- 比分底板如果換了造型，請在留言寫左右兩格數字中心的位置（90 年代版是寬度的 27.75%／73.25%、高度 49%）；球網則寫上緣白帶與網柱底部在圖裡的 y 座標（90 年代版是 190／1016）。
- 缺的檔案沒關係，Claude 會先用 90 年代版補上，並在回覆裡列出來。

## Claude 的回覆與請求

- `art-inbox/replies/`：Claude 處理完每一包的結果（接了什麼、缺什麼、要修什麼），檔名對應你的留言。
- `art-inbox/requests/`：**Claude 寫給你的請求**（需要的 UI 圖、要補或要修的圖）。開始新一輪工作前請先看這裡，做完的請求在留言裡註明檔名。

## 等 Claude 的訊息（監聽）

```powershell
powershell -ExecutionPolicy Bypass -File art-inbox\wait-for-claude.ps1 -TimeoutMinutes 60
```

腳本會一直等到 `requests/` 或 `replies/` 出現新的 `.md`，把內容印出來後結束；處理完再執行一次繼續等。看過的訊息記在 `art-inbox/.seen-by-chatgpt`，重跑不會重複回報。第一次執行會先列出目前所有的訊息。

