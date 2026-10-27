# 🗡️ 虛擬闖天下 Virtual Quest

《雙截龍》《吞食天地》風格的清版動作遊戲（小朋友齊打交），主角是[虛擬台麻館](https://github.com/YuutaTsubasa/TaiwanMahjong)的角色們：小翠、月華、阿明、糖糖、小春、蘭姐、蒼牙、金爺、雷歐、阿烈，還有隱藏角色阿翼。最多四個人一起闖六大關。跟[虛擬排球王](https://github.com/YuutaTsubasa/VirtualVolleyball)同一套做法：Tauri v2 + Svelte 5，遊戲畫面用 Canvas 畫，遊戲引擎用 Rust 寫（網頁版編成 WebAssembly）。

## 六大關

| | 大關 | 小關 1 | 小關 2 | 小關 3（魔王） |
| --- | --- | --- | --- | --- |
| 1 | 白色都市 | 白色大道 | 天台夜路 | 中央廣場：白銀幫老大（衝撞） |
| 2 | 草原森林 | 風之草原 | 迷霧森林 | 巨木深處：森林巨熊（跳起來砸地） |
| 3 | 水下世界 | 珊瑚礁 | 沉船 | 深海神殿：鯊魚將軍（衝撞＋泡泡） |
| 4 | 冰山 | 雪原 | 冰窟 | 冰之王座：冰雪女王（冰柱＋跳躍） |
| 5 | 火炎礦坑 | 礦坑入口 | 熔岩河 | 火山心臟：熔岩巨人（砸地＋火球＋衝撞） |
| 6 | 邪惡城堡 | 城門 | 迴廊 | 魔王之間：魔王（瞬移＋暗黑彈＋召喚） |

- 每一小關是往右捲的路，路上有幾個「關卡鎖」：敵人從左右進場，全部打倒才會出現「GO ▶」。小關 1、2 的最後有強化的小頭目。
- 敵人：小混混（出拳）、刀手（快、打完就退）、大隻佬（慢、血厚、會衝撞）、投擲手（保持距離丟東西），每一大關換造型與名字。
- 打破路上的箱子會掉出蘋果、烤肉、大餐（補血）或寶石（分數）。
- 全軍覆沒可以接關（從這一小關重來）；打過的大關下次可以直接選來開始。
- 人越多敵人越多、魔王血越厚；難度（簡單／普通／困難）影響敵人的攻擊力與出手頻率。

## 怎麼玩

| | 鍵盤 1P | 鍵盤 2P | 手把 | 觸控 |
| --- | --- | --- | --- | --- |
| 移動 | W A S D | 方向鍵 | 十字鍵／左搖桿 | 左下搖桿 |
| 攻擊 | J | 數字鍵 1／, | X／RB | 「攻擊」 |
| 跳 | K | 數字鍵 2／. | A／LB | 「跳」 |
| 必殺 | L | 數字鍵 3／/ | B／Y／RT | 「必殺」 |
| 暫停 | Esc／P | Esc／P | Start | 上方暫停鈕 |

- **連段**：連按攻擊，打中後接第二、三下，第三下會把敵人踢倒。
- **跳踢**：跳起來在空中按攻擊。
- **抓摔**：敵人被打得搖晃時走過去就會抓住；按攻擊膝擊，第三下（或反方向＋攻擊）丟出去，能撞倒其他敵人。
- **必殺**：打倒身邊所有敵人（攻擊＋跳一起按也可以），打中會扣一點自己的體力。
- 一個人玩時方向鍵＋Z／X／C 也可以。多人同樂時第 1 支手把是 1P、第 2 支是 2P，依此類推（鍵盤最多兩人，手把最多四人）。
- 打倒第一大關的魔王會解鎖隱藏角色「阿翼」——或在選單輸入 ↑↑↓↓←→←→BA。

模式：

- **一人闖關**：1P＋最多三位電腦隊友。
- **多人同樂**：同一台電腦 2～4 人（可再加電腦隊友補到四人）。
- **電腦示範**：看電腦隊伍闖關。
- **開設房間／加入房間**：線上（5 碼房號、分享連結 `…/#join/房號`）或區網，最多四人，房主可以加電腦隊友；斷線時由電腦代打，重連就還給你。

## 開發

需求：Node 18+、Rust 1.89+、`rustup target add wasm32-unknown-unknown`。

```bash
npm install
```

網頁版（http://localhost:1422）：

```bash
npm run dev
```

桌面 App：

```bash
npm run tauri dev
```

不開視窗的區網伺服器（先 `npm run build`），會印出網址與「房主金鑰」：

```bash
npm run server -- --port 7799 --dist dist --world 1
```

測試（JS 的漏翻、介面檢查＋Rust 的遊戲與房間測試，包含「電腦隊伍每一大關都打得完、不會卡住」）：

```bash
npm test
```

看電腦隊伍能打到哪一關（平衡用）：

```bash
cargo test --manifest-path crates/quest-core/Cargo.toml --release balance -- --ignored --nocapture
```

## 連線

跟排球王一樣：遊戲在房主那邊跑（每秒 60 格），玩家只送輸入；房間與訊息的處理（`crates/quest-core/src/protocol.rs`）同一份 Rust 程式給區網伺服器、線上房間（WASM，在房主瀏覽器的 Web Worker 裡）與單機用。線上房間用 WebRTC，每條連線一個可靠通道（大廳、指令、聊天）與一個不可靠通道（影格、輸入）。輸入的「按下」用累計次數送，掉封包也不會漏按。

### 配對服務（Cloudflare Workers）

程式在 [VirtualCore](https://github.com/YuutaTsubasa/VirtualCore) 的 `workers/signal/`（Virtual 系列五款共用），本作部署的名字是 `virtual-quest-signal`。部署要用你的 Cloudflare 帳號登入（第一次 `npx wrangler login`）：

```bash
# 在 VirtualCore 的資料夾
npm --prefix workers/signal run deploy:quest
```

已部署在 `https://virtual-quest-signal.taiwan-mahjong-signal.workers.dev`（`src/lib/site.js` 的預設值；打開 `/health` 會看到 `ok`）。配對服務連不上時，介面會自動藏起線上房間。

本機測試（VirtualCore clone 在旁邊的 `../VirtualCore`）：`npm --prefix ../VirtualCore/workers/signal run dev`（http://localhost:8787），在 `.env.development.local` 寫 `VITE_SIGNAL_SERVER=http://localhost:8787`。

區網房間預設通訊埠 **7799**（台麻 7777、排球 7788，可以同時開）。

## 程式結構

```
crates/quest-core/       遊戲引擎（Rust）
  src/game.rs              清版動作模擬：玩家、敵人、魔王招式、箱子、道具、飛行道具、鏡頭與關卡鎖
  src/stages.rs            六大關 × 三小關的關卡表（每波敵人、箱子）
  src/ai.rs                電腦隊友
  src/room.rs              房間：四個座位、電腦隊友、聊天、接關
  src/protocol.rs          連線訊息（區網伺服器與線上房間共用）
crates/quest-wasm/       WebAssembly：單機（LocalMatch）與線上房主（HostRoom）
src-tauri/               Tauri v2 App＋區網伺服器（axum，/ws）
src/
  App.svelte               畫面切換（標題→模式→選角→闖關→過關／全軍覆沒／破關）
  lib/engine.js            主迴圈
  lib/game/render.js       Canvas：角色、敵人、箱子、道具、特效、HUD
  lib/game/scenery.js      六大關的背景（程式畫的，三層視差）
  lib/game/stages.js       關名、敵人名字與配色
  lib/game/input.js        鍵盤＋手把＋觸控
  lib/game/characters.js   角色名字、台詞、圖
```

### 跟 VirtualCore 共用的部分

[VirtualCore](https://github.com/YuutaTsubasa/VirtualCore) 是 Virtual 系列五款共用的程式，`package.json` 用 git tag 引用（`virtual-core`）：

- `virtual-core/audio/synth.js`、`audio/music.js`：PC-98 FM 音源與曲子播放器（曲目是本作的 `src/lib/audio/songs.js`）
- `virtual-core/net/p2p.js`、`net/site.js`：線上房間的 WebRTC 傳輸、房號與分享連結（本作的網址在 `src/lib/site.js`）
- `workers/signal/`：配對服務
- `.github/workflows/build-apps.yml`、`pages.yml`：這個 repo 的兩個 workflow 只是呼叫它們

本機要同時改 VirtualCore 時：`npm install ../VirtualCore` 暫時換成本機版本（不要 commit 這個改動）。

## 美術

- 介面照 [UI Concept v1](art-work/ui-concept-v1/DESIGN.md)：墨黑＋暖白＋朱紅＋青綠、直角實色陰影、毛筆大標題。Logo 是那裡的 `logo-transparent.png`。
- 角色半身像沿用虛擬台麻館的（麻將館的衣服）。
- 動作圖、敵人、魔王、背景由 ChatGPT 透過 `art-inbox/` 交件，用 `python scripts/import-quest-art.py art-inbox/<包>` 接進遊戲（產生圖集與 `src/lib/game/sheets.json`）。還沒交的部分程式會用替代畫法（角色借排球王的 8 格動作圖、敵人與背景用程式畫）。
- 已接：小春 16 格、白色都市的混混與白銀幫老大、1-1 背景、箱子與道具。其他在 `art-inbox/requests/` 排隊中。
