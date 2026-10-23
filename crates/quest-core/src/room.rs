//! 房間：四個座位（真人或電腦隊友）、觀戰者、聊天、開始／接關／回大廳。
//! 不碰網路，伺服器（src-tauri/src/server.rs）每 1/60 秒呼叫 `tick()`。

use crate::ai::{Ai, Difficulty};
use crate::characters::{is_character, random_character};
use crate::game::{Entrant, Event, Input, Match, MAX_PLAYERS};
use crate::stages::WORLDS;
use rand::distributions::Alphanumeric;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SeatKind {
    Human,
    Bot,
}

pub struct Seat {
    pub name: String,
    pub character: String,
    pub kind: SeatKind,
    pub difficulty: Difficulty,
    pub token: String,
    pub connected: bool,
    input: Input,
    /// 攻擊／跳／必殺「按下」的累計次數（各 8 位元），跟上次不同就是有按
    press_seq: u64,
    pending: u32,
}

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

#[derive(Clone, Debug, Serialize)]
struct ChatLine {
    seat: Option<usize>,
    name: String,
    text: String,
    sys: bool,
}

const SEAT_NAMES: [&str; MAX_PLAYERS] = ["1P", "2P", "3P", "4P"];
const MAX_CHAT: usize = 60;

pub struct Room {
    pub seats: [Option<Seat>; MAX_PLAYERS],
    pub host_seat: Option<usize>,
    pub settings: Settings,
    pub game: Option<Match>,
    ais: [Option<Ai>; MAX_PLAYERS],
    chat: Vec<ChatLine>,
    rng: SmallRng,
}

impl Default for Room {
    fn default() -> Self {
        Self::new(Settings::default())
    }
}

impl Room {
    pub fn new(settings: Settings) -> Self {
        Room {
            seats: [None, None, None, None],
            host_seat: None,
            settings,
            game: None,
            ais: [None, None, None, None],
            chat: Vec::new(),
            rng: SmallRng::from_entropy(),
        }
    }

    pub fn playing(&self) -> bool {
        self.game.is_some()
    }

    fn sys(&mut self, text: String) {
        self.chat.push(ChatLine { seat: None, name: String::new(), text, sys: true });
        if self.chat.len() > MAX_CHAT {
            self.chat.remove(0);
        }
    }

    pub fn push_chat(&mut self, seat: Option<usize>, text: &str) {
        let text: String = text.trim().chars().take(300).collect();
        if text.is_empty() {
            return;
        }
        let name = seat
            .and_then(|s| self.seats[s].as_ref())
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "觀眾".into());
        self.chat.push(ChatLine { seat, name, text, sys: false });
        if self.chat.len() > MAX_CHAT {
            self.chat.remove(0);
        }
    }

    fn pick_character(&mut self, wanted: Option<&str>) -> String {
        match wanted {
            Some(c) if is_character(c) => c.to_string(),
            _ => random_character(&mut self.rng, &[]).to_string(),
        }
    }

    /// 真人入席，回傳（座位, 重新連線用的 token）。比賽中不能入席（改成觀戰）。
    pub fn join(&mut self, name: &str, character: Option<&str>) -> Result<(usize, String), String> {
        if self.playing() {
            return Err("比賽進行中".into());
        }
        let seat = self.seats.iter().position(|s| s.is_none()).ok_or("座位已滿")?;
        let name: String = name.trim().chars().take(16).collect();
        let name = if name.is_empty() { format!("玩家{}", seat + 1) } else { name };
        let token: String = (&mut self.rng).sample_iter(&Alphanumeric).take(24).map(char::from).collect();
        let character = self.pick_character(character);
        self.sys(format!("{} 坐到了 {}", name, SEAT_NAMES[seat]));
        self.seats[seat] = Some(Seat {
            name,
            character,
            kind: SeatKind::Human,
            difficulty: self.settings.difficulty,
            token: token.clone(),
            connected: false,
            input: Input::default(),
            press_seq: 0,
            pending: 0,
        });
        Ok((seat, token))
    }

    pub fn seat_of_token(&self, token: &str) -> Option<usize> {
        if token.is_empty() {
            return None;
        }
        self.seats
            .iter()
            .position(|s| s.as_ref().is_some_and(|s| s.kind == SeatKind::Human && s.token == token))
    }

    pub fn connect(&mut self, seat: usize) {
        if let Some(s) = self.seats[seat].as_mut() {
            s.connected = true;
        }
    }

    /// 斷線：比賽中由電腦代打，重新連上就還給玩家
    pub fn disconnect(&mut self, seat: usize) {
        let Some(s) = self.seats[seat].as_mut() else { return };
        s.connected = false;
        s.input = Input::default();
        let name = s.name.clone();
        if self.playing() {
            self.sys(format!("{} 斷線了，先由電腦代打", name));
        }
    }

    pub fn add_bot(&mut self, seat: Option<usize>, difficulty: Difficulty) -> Result<usize, String> {
        if self.playing() {
            return Err("比賽進行中".into());
        }
        let seat = match seat {
            Some(s) if s < MAX_PLAYERS && self.seats[s].is_none() => s,
            Some(_) => return Err("這個座位有人了".into()),
            None => self.seats.iter().position(|s| s.is_none()).ok_or("座位已滿")?,
        };
        let taken: Vec<String> = self.seats.iter().flatten().map(|s| s.character.clone()).collect();
        let taken: Vec<&str> = taken.iter().map(|s| s.as_str()).collect();
        let character = random_character(&mut self.rng, &taken).to_string();
        self.seats[seat] = Some(Seat {
            name: format!("電腦隊友（{}）", difficulty.label()),
            character,
            kind: SeatKind::Bot,
            difficulty,
            token: String::new(),
            connected: true,
            input: Input::default(),
            press_seq: 0,
            pending: 0,
        });
        Ok(seat)
    }

    /// 離開座位（玩家自己離開或房主移除）。比賽中就換成電腦接手。
    pub fn remove_seat(&mut self, seat: usize) -> Result<(), String> {
        let Some(s) = self.seats.get_mut(seat).and_then(|s| s.as_mut()) else {
            return Err("這個座位沒有人".into());
        };
        let name = s.name.clone();
        if self.game.is_some() {
            s.kind = SeatKind::Bot;
            s.token.clear();
            s.connected = true;
            s.name = format!("電腦（代打 {}）", name);
        } else {
            self.seats[seat] = None;
        }
        if self.host_seat == Some(seat) {
            self.host_seat = None;
        }
        self.sys(format!("{} 離開了座位", name));
        Ok(())
    }

    pub fn set_character(&mut self, seat: usize, id: &str) -> Result<(), String> {
        if self.playing() {
            return Err("比賽進行中不能換角色".into());
        }
        if !is_character(id) {
            return Err("沒有這個角色".into());
        }
        let s = self.seats[seat].as_mut().ok_or("你沒有座位")?;
        s.character = id.to_string();
        Ok(())
    }

    pub fn update_settings(&mut self, settings: Settings) {
        self.settings = Settings { world: settings.world.min(WORLDS - 1), lives: settings.lives.clamp(1, 9), ..settings };
    }

    /// 玩家輸入：bits 是方向與攻擊按住（見 `Input::from_bits` 的低 5 位元），
    /// seq 是攻擊／跳／必殺「按下」的累計次數（各 8 位元）。用次數而不是「剛按下」，
    /// 中間掉了幾個封包也不會漏掉按鍵。
    pub fn set_input(&mut self, seat: usize, bits: u32, seq: u64) {
        let Some(s) = self.seats.get_mut(seat).and_then(|s| s.as_mut()) else { return };
        s.input = Input::from_bits(bits & 0b1_1111);
        for (k, bit) in [(0, 32u32), (8, 64), (16, 128)] {
            if (seq >> k) & 0xff != (s.press_seq >> k) & 0xff {
                s.pending |= bit;
            }
        }
        s.press_seq = seq;
    }

    /// 出發：有人的座位都上場（空位不補電腦，要隊友請房主加）
    pub fn start(&mut self) -> Result<(), String> {
        let entrants: Vec<Entrant> = self
            .seats
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|s| Entrant { slot: i, character: &s.character }))
            .collect();
        if entrants.is_empty() {
            return Err("沒有人上場".into());
        }
        let seed = self.rng.gen();
        self.game = Some(Match::new(&entrants, self.settings.world, self.settings.difficulty, self.settings.lives, seed));
        self.ais = [None, None, None, None];
        self.sys("出發！".into());
        Ok(())
    }

    /// 全軍覆沒後接關
    pub fn continue_game(&mut self) -> Result<(), String> {
        let game = self.game.as_mut().ok_or("還沒開始")?;
        if !game.continue_game() {
            return Err("現在不能接關".into());
        }
        self.sys("接關！".into());
        Ok(())
    }

    pub fn reset_to_lobby(&mut self) {
        self.game = None;
        self.ais = [None, None, None, None];
    }

    /// 前進一個影格。比賽中回傳 true。
    pub fn tick(&mut self) -> bool {
        let Some(game) = self.game.as_mut() else { return false };
        let mut inputs = Vec::with_capacity(game.players.len());
        for (idx, p) in game.players.iter().enumerate() {
            let i = p.slot;
            let Some(seat) = self.seats[i].as_mut() else {
                inputs.push(Input::default());
                continue;
            };
            if seat.kind == SeatKind::Human && seat.connected {
                self.ais[i] = None;
                let mut input = seat.input;
                let pressed = std::mem::take(&mut seat.pending);
                input.attack_pressed = pressed & 32 != 0;
                input.jump_pressed = pressed & 64 != 0;
                input.special_pressed = pressed & 128 != 0;
                inputs.push(input);
            } else {
                let difficulty = seat.difficulty;
                let seed = self.rng.gen();
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
            self.sys(l);
        }
        true
    }

    /// 大廳／房間資訊（有變動時送）
    pub fn view(&self, you: Option<usize>) -> Value {
        let seats: Vec<Value> = self
            .seats
            .iter()
            .map(|s| {
                s.as_ref().map_or(Value::Null, |s| {
                    json!({
                        "name": s.name,
                        "character": s.character,
                        "kind": s.kind,
                        "difficulty": s.difficulty,
                        "connected": s.connected,
                    })
                })
            })
            .collect();
        json!({
            "phase": if self.playing() { "playing" } else { "lobby" },
            "seats": seats,
            "host_seat": self.host_seat,
            "you": you,
            "settings": self.settings,
            "chat": self.chat,
        })
    }

    /// 比賽畫面（每個影格送）
    pub fn frame(&self) -> Option<Value> {
        self.game.as_ref().and_then(|g| serde_json::to_value(g).ok())
    }
}
