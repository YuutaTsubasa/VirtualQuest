//! 房間：四個座位（真人或電腦隊友）、觀戰者、聊天、開始／接關／回大廳。
//! 座位、房主、聊天這些共通的部分在 VirtualCore 的 virtual-room（`Lobby`），這裡只放闖天下的比賽。
//! 不碰網路，伺服器（VirtualCore 的 virtual-lan）每 1/60 秒呼叫 `tick()`。

use crate::ai::{Ai, Difficulty};
use crate::characters::{is_character, random_character};
use crate::game::{Entrant, Event, Input, Match, MAX_PLAYERS};
use crate::stages::WORLDS;
use rand::rngs::SmallRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ops::{Deref, DerefMut};
use virtual_room::{Game, Lobby, RoomGame};

pub use virtual_room::{Seat, SeatKind};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// 難度（敵人強度，也是電腦隊友的強度）
    pub difficulty: Difficulty,
    /// 從第幾大關開始（0 起算）
    pub world: usize,
    /// 每人幾條命
    pub lives: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { difficulty: Difficulty::Normal, world: 0, lives: 3 }
    }
}

/// 每個座位的輸入
#[derive(Default)]
pub struct SeatInput {
    input: Input,
    /// 攻擊／跳／必殺「按下」的累計次數（各 8 位元），跟上次不同就是有按
    press_seq: u64,
    pending: u32,
}

const SEAT_NAMES: [&str; MAX_PLAYERS] = ["1P", "2P", "3P", "4P"];

/// 闖天下的房間規則
pub struct Quest;

impl Game for Quest {
    type Settings = Settings;
    type SeatState = SeatInput;
    const SEATS: usize = MAX_PLAYERS;

    fn joined_text(name: &str, seat: usize) -> String {
        format!("{} 坐到了 {}", name, SEAT_NAMES[seat])
    }
    fn bot_name(difficulty: Difficulty) -> String {
        format!("電腦隊友（{}）", difficulty.label())
    }
    fn is_character(id: &str) -> bool {
        is_character(id)
    }
    fn random_character(rng: &mut SmallRng, avoid: &[&str]) -> &'static str {
        random_character(rng, avoid)
    }
    fn clamp_settings(settings: Settings) -> Settings {
        Settings { world: settings.world.min(WORLDS - 1), lives: settings.lives.clamp(1, 9), ..settings }
    }
    fn seat_difficulty(settings: &Settings) -> Difficulty {
        settings.difficulty
    }
    fn on_disconnect(state: &mut SeatInput) {
        state.input = Input::default();
    }
}

pub struct Room {
    pub lobby: Lobby<Quest>,
    pub game: Option<Match>,
    ais: [Option<Ai>; MAX_PLAYERS],
}

impl Default for Room {
    fn default() -> Self {
        Self::new(Settings::default())
    }
}

// 座位、聊天、入席這些直接用大廳的（room.join(…)、room.seats…）
impl Deref for Room {
    type Target = Lobby<Quest>;
    fn deref(&self) -> &Lobby<Quest> {
        &self.lobby
    }
}

impl DerefMut for Room {
    fn deref_mut(&mut self) -> &mut Lobby<Quest> {
        &mut self.lobby
    }
}

impl Room {
    pub fn new(settings: Settings) -> Self {
        Room { lobby: Lobby::new(settings), game: None, ais: [None, None, None, None] }
    }

    /// 玩家輸入：bits 是方向與攻擊按住（見 `Input::from_bits` 的低 5 位元），
    /// seq 是攻擊／跳／必殺「按下」的累計次數（各 8 位元）。用次數而不是「剛按下」，
    /// 中間掉了幾個封包也不會漏掉按鍵。
    pub fn set_input(&mut self, seat: usize, bits: u32, seq: u64) {
        let Some(s) = self.lobby.seats.get_mut(seat).and_then(|s| s.as_mut()) else { return };
        let s = &mut s.state;
        s.input = Input::from_bits(bits & 0b1_1111);
        for (k, bit) in [(0, 32u32), (8, 64), (16, 128)] {
            if (seq >> k) & 0xff != (s.press_seq >> k) & 0xff {
                s.pending |= bit;
            }
        }
        s.press_seq = seq;
    }

    /// 全軍覆沒後接關
    pub fn continue_game(&mut self) -> Result<(), String> {
        let game = self.game.as_mut().ok_or("還沒開始")?;
        if !game.continue_game() {
            return Err("現在不能接關".into());
        }
        self.lobby.sys("接關！");
        Ok(())
    }
}

impl RoomGame for Room {
    type Rules = Quest;

    fn lobby(&self) -> &Lobby<Quest> {
        &self.lobby
    }
    fn lobby_mut(&mut self) -> &mut Lobby<Quest> {
        &mut self.lobby
    }

    /// 出發：有人的座位都上場（空位不補電腦，要隊友請房主加）
    fn start(&mut self) -> Result<(), String> {
        let entrants: Vec<Entrant> = self
            .lobby
            .seats
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|s| Entrant { slot: i, character: &s.character }))
            .collect();
        if entrants.is_empty() {
            return Err("沒有人上場".into());
        }
        let seed = self.lobby.rng.gen();
        let s = &self.lobby.settings;
        let game = Match::new(&entrants, s.world, s.difficulty, s.lives, seed);
        self.game = Some(game);
        self.ais = [None, None, None, None];
        self.lobby.playing = true;
        self.lobby.sys("出發！");
        Ok(())
    }

    fn reset_to_lobby(&mut self) {
        self.game = None;
        self.ais = [None, None, None, None];
        self.lobby.playing = false;
    }

    fn input(&mut self, seat: usize, msg: &Value) {
        let bits = msg.get("b").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
        let seq = msg.get("s").and_then(|x| x.as_u64()).unwrap_or(0);
        self.set_input(seat, bits, seq);
    }

    fn host_command(&mut self, cmd: &str, _msg: &Value) -> Option<Result<(), String>> {
        (cmd == "continue").then(|| self.continue_game())
    }

    /// 前進一個影格。比賽中回傳 true（每格都送畫面）。
    fn tick(&mut self) -> bool {
        let Some(game) = self.game.as_mut() else { return false };
        let mut inputs = Vec::with_capacity(game.players.len());
        for (idx, p) in game.players.iter().enumerate() {
            let i = p.slot;
            let Some(seat) = self.lobby.seats[i].as_mut() else {
                inputs.push(Input::default());
                continue;
            };
            if seat.is_live_human() {
                self.ais[i] = None;
                let mut input = seat.state.input;
                let pressed = std::mem::take(&mut seat.state.pending);
                input.attack_pressed = pressed & 32 != 0;
                input.jump_pressed = pressed & 64 != 0;
                input.special_pressed = pressed & 128 != 0;
                inputs.push(input);
            } else {
                let difficulty = seat.difficulty;
                let seed = self.lobby.rng.gen();
                inputs.push(self.ais[i].get_or_insert_with(|| Ai::new(idx, difficulty, seed)).think(game));
            }
        }
        game.step(&inputs);
        let mut lines = Vec::new();
        for e in &game.events {
            match e {
                Event::Clear { world, sub } => lines.push(format!("第 {}-{} 關突破！", world + 1, sub + 1)),
                Event::Over => lines.push("全軍覆沒……".to_string()),
                Event::Ending => lines.push("六大關全部突破！".to_string()),
                _ => {}
            }
        }
        for l in lines {
            self.lobby.sys(l);
        }
        true
    }

    /// 比賽畫面（每個影格送）
    fn frame(&self) -> Option<Value> {
        self.game.as_ref().and_then(|g| serde_json::to_value(g).ok())
    }

    /// 這一格有過關／全軍覆沒／破關（大廳資訊要更新：聊天多了一行系統訊息）
    fn just_ended(&mut self) -> bool {
        self.game
            .as_ref()
            .is_some_and(|g| g.events.iter().any(|e| matches!(e, Event::Over | Event::Clear { .. } | Event::Ending)))
    }
}
