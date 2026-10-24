//! 清版動作的模擬：玩家、敵人、箱子、道具、飛行道具與鏡頭。每次 `step()` 前進固定 1/60 秒。
//! 伺服器（連線）與 WebAssembly（單機）都跑這一份，前端只負責畫。
//!
//! 座標：x 是關卡裡的橫向位置（往右捲），z 是「地面在畫面上的高度」（越大越靠近鏡頭），
//! y 是離地高度（往上為正）。畫面位置 = (x − 鏡頭, z − y)。
//! 欄位名稱用 camelCase 序列化，前端 src/lib/game/render.js 直接讀。

use crate::ai::Difficulty;
use crate::characters::{special_of, stats_of, Reach, Special, Stats};
use crate::stages::{build_stage, Spawn, StageDef, SUBS, WORLDS};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;

pub const W: f32 = 960.0;
pub const H: f32 = 540.0;
/// 可以走的地面：畫面上 z 的範圍
pub const Z_TOP: f32 = 340.0;
pub const Z_BOTTOM: f32 = 520.0;
pub const MAX_PLAYERS: usize = 4;

const G: f32 = 0.6;
const JUMP_V: f32 = 10.5;
/// 前後（z）方向走得比左右慢
const Z_SPEED: f32 = 0.62;
/// 同時在畫面上的敵人上限
const ENEMY_CAP: usize = 9;
const INTRO_FRAMES: i32 = 140;
const CLEAR_FRAMES: i32 = 200;
const RESPAWN_FRAMES: i32 = 70;
/// 一次能有幾隻敵人圍著同一個玩家打（其他的在旁邊繞）
const ATTACKERS_PER_PLAYER: usize = 2;
/// 跑步：連按兩下左右，第二下要在這麼多格以內
const DOUBLE_TAP: i32 = 14;
/// 跑步比走路快幾倍
const RUN_SPEED: f32 = 1.9;

/// 一個玩家在這個影格的輸入
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    /// 攻擊鍵按住中（電腦隊友不用，給以後的蓄力用）
    pub attack: bool,
    pub attack_pressed: bool,
    pub jump_pressed: bool,
    pub special_pressed: bool,
}

impl Input {
    /// 前端傳來的位元：1 左、2 右、4 上、8 下、16 攻擊（按住）、32 攻擊（剛按下）、64 跳（剛按下）、128 必殺（剛按下）
    pub fn from_bits(b: u32) -> Self {
        Input {
            left: b & 1 != 0,
            right: b & 2 != 0,
            up: b & 4 != 0,
            down: b & 8 != 0,
            attack: b & 16 != 0,
            attack_pressed: b & 32 != 0,
            jump_pressed: b & 64 != 0,
            special_pressed: b & 128 != 0,
        }
    }
    fn dx(&self) -> f32 {
        (self.right as i32 - self.left as i32) as f32
    }
    fn dz(&self) -> f32 {
        (self.down as i32 - self.up as i32) as f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Act {
    Stand,
    Walk,
    /// 跑步（連按兩下左右）
    Run,
    Attack,
    Jump,
    Hurt,
    Fall,
    Down,
    Rise,
    /// 抓住敵人
    Grab,
    /// 被抓住
    Held,
    /// 被丟出去（會撞倒別的敵人）
    Thrown,
    Dead,
    Win,
    /// 從畫面上方掉下來（玩家重新上場）
    Enter,
    /// 衝撞（大隻佬、魔王）
    Charge,
    /// 跳起來砸地（魔王）
    Leap,
    /// 放飛行道具
    Cast,
    /// 瞬間移動（最終魔王）
    Vanish,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Move {
    None,
    Jab,
    Jab2,
    Kick,
    /// 跑步中按攻擊：往前衝的一擊（會把敵人撞倒）
    DashAttack,
    JumpKick,
    Special,
    Knee,
    Throw,
    Slash,
    Slam,
    BossSlam,
    Uppercut,
    /// 雜兵的拳頭（比玩家的刺拳慢很多）
    Punch,
    /// 拿著武器揮
    Bat,
    Blade,
    Shoot,
}

#[derive(Clone, Copy, Debug)]
struct MoveData {
    startup: i32,
    active: i32,
    recover: i32,
    dmg: f32,
    /// 打得到的範圍（面向的方向，離身體多遠）
    near: f32,
    far: f32,
    /// 前後（z）差多少以內打得到
    zr: f32,
    knock: bool,
    /// 前後都打得到（必殺技）
    omni: bool,
}

const fn md(startup: i32, active: i32, recover: i32, dmg: f32, far: f32, zr: f32, knock: bool) -> MoveData {
    MoveData { startup, active, recover, dmg, near: -6.0, far, zr, knock, omni: false }
}

fn move_data(m: Move) -> MoveData {
    match m {
        Move::None | Move::Throw | Move::Shoot => md(0, 0, 18, 0.0, 0.0, 0.0, false),
        Move::Jab => md(4, 3, 8, 6.0, 70.0, 28.0, false),
        Move::Jab2 => md(4, 3, 9, 7.0, 74.0, 28.0, false),
        // 連段收尾：單挑時傷害最高的打法
        Move::Kick => md(6, 4, 15, 14.0, 92.0, 30.0, true),
        // 衝刺攻擊：打得久一點，衝過一整排都打得到（打空的話收招會更久，見 update_players）
        Move::DashAttack => md(3, 12, 16, 12.0, 82.0, 30.0, true),
        // 跳踢：接近、脫身用，傷害比連段低
        Move::JumpKick => md(3, 60, 0, 9.0, 78.0, 30.0, true),
        // 魔王的對空上勾拳：看到玩家在身邊跳起來就打下來
        Move::Uppercut => md(5, 7, 18, 14.0, 100.0, 26.0, true),
        Move::Special => MoveData { omni: true, ..md(3, 14, 14, 14.0, 100.0, 24.0, true) },
        Move::Knee => md(0, 1, 9, 8.0, 50.0, 20.0, false),
        // 雜兵的拳頭：起手 12 格，手會先往後拉（看到就先打他，打中就能打斷）
        Move::Punch => md(12, 4, 16, 10.0, 64.0, 16.0, false),
        // 武器：鈍器一揮打倒、範圍長；刀快、可以一直砍（第四下會把人砍倒）
        Move::Bat => md(7, 5, 14, 16.0, 104.0, 30.0, true),
        Move::Blade => md(4, 3, 9, 10.0, 88.0, 28.0, false),
        Move::Slash => md(10, 3, 20, 12.0, 78.0, 15.0, false),
        Move::Slam => md(17, 5, 24, 15.0, 86.0, 20.0, true),
        Move::BossSlam => md(15, 5, 22, 17.0, 104.0, 24.0, true),
    }
}

impl Move {
    fn total(self) -> i32 {
        let d = move_data(self);
        d.startup + d.active + d.recover
    }
}

/// 玩家與敵人共用的身體：位置、體力、動作
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    pub x: f32,
    pub z: f32,
    pub y: f32,
    #[serde(skip)]
    pub vx: f32,
    #[serde(skip)]
    pub vz: f32,
    #[serde(skip)]
    pub vy: f32,
    /// 面向：1 右、-1 左
    pub facing: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub act: Act,
    /// 進入目前動作後經過的影格（畫面挑動作圖用）
    pub t: i32,
    pub mv: Move,
    /// 無敵剩餘影格（畫面會閃爍）
    pub inv: i32,
    /// 剛被打中的白色閃光
    pub flash: i32,
    #[serde(skip)]
    pub dur: i32,
    /// 硬直中又被打了幾下（太多下就倒地，避免無限連段）
    #[serde(skip)]
    pub stun: i32,
    /// 這一招已經打中過誰（每招每個目標只算一次）
    #[serde(skip)]
    pub hit_list: Vec<u32>,
    /// 這一招有沒有打中（連段用）
    #[serde(skip)]
    pub connected: bool,
    /// 抓住的敵人 id（玩家）／抓住自己的玩家 slot（敵人）
    #[serde(skip)]
    pub hold: Option<u32>,
}

impl Body {
    fn new(x: f32, z: f32, hp: i32, facing: f32) -> Self {
        Body {
            x,
            z,
            y: 0.0,
            vx: 0.0,
            vz: 0.0,
            vy: 0.0,
            facing,
            hp,
            max_hp: hp,
            act: Act::Stand,
            t: 0,
            mv: Move::None,
            inv: 0,
            flash: 0,
            dur: 0,
            stun: 0,
            hit_list: Vec::new(),
            connected: false,
            hold: None,
        }
    }

    fn set(&mut self, act: Act, dur: i32) {
        self.act = act;
        self.t = 0;
        self.dur = dur;
        self.hit_list.clear();
        self.connected = false;
        if !matches!(act, Act::Attack | Act::Jump) {
            self.mv = Move::None;
        }
    }

    fn attack(&mut self, mv: Move) {
        self.set(Act::Attack, mv.total());
        self.mv = mv;
        self.vx = 0.0;
        self.vz = 0.0;
    }

    /// 打得到（倒地、起身、無敵中都打不到）
    pub fn vulnerable(&self) -> bool {
        self.inv == 0 && !matches!(self.act, Act::Fall | Act::Down | Act::Rise | Act::Dead | Act::Thrown | Act::Vanish | Act::Enter | Act::Win)
    }

    pub fn alive(&self) -> bool {
        self.act != Act::Dead
    }

    /// 這一格是不是招式的有效時間，是的話回傳招式資料
    fn active_move(&self) -> Option<MoveData> {
        let d = move_data(self.mv);
        match (self.act, self.mv) {
            (Act::Jump, Move::JumpKick) => Some(d),
            (Act::Attack, m) if m != Move::None && m != Move::Knee && m != Move::Throw && m != Move::Shoot => {
                (self.t >= d.startup && self.t < d.startup + d.active).then_some(d)
            }
            _ => None,
        }
    }

    /// r：對方身體的半寬（從身體中心到邊緣）
    fn reaches(&self, d: &MoveData, x: f32, z: f32, y: f32, r: f32) -> bool {
        if (z - self.z).abs() > d.zr || (y - self.y).abs() > 90.0 {
            return false;
        }
        if d.omni {
            return (x - self.x).abs() <= d.far + r;
        }
        let ahead = (x - self.x) * self.facing;
        ahead >= d.near - r && ahead <= d.far + r
    }

    /// 空中的移動與落地；回傳這一格是不是剛落地
    fn air(&mut self) -> bool {
        if self.y <= 0.0 && self.vy <= 0.0 {
            return false;
        }
        self.y += self.vy;
        self.vy -= G;
        self.x += self.vx;
        self.z += self.vz;
        if self.y <= 0.0 {
            self.y = 0.0;
            self.vy = 0.0;
            return true;
        }
        false
    }

    /// 被打中：扣血、硬直或倒地。dir 是攻擊者往哪邊打
    fn take_hit(&mut self, dmg: i32, knock: bool, dir: f32, armor: bool) {
        self.hp = (self.hp - dmg).max(0);
        self.flash = 7;
        if self.hold.is_some() && self.act == Act::Held {
            self.hold = None;
        }
        if armor && self.hp > 0 {
            return;
        }
        self.facing = -dir;
        if self.hp <= 0 || knock || self.stun >= 3 || self.y > 0.0 {
            self.set(Act::Fall, 0);
            self.vy = 6.0;
            self.vx = dir * 3.6;
            self.vz = 0.0;
            self.y = self.y.max(1.0);
            self.stun = 0;
        } else {
            self.set(Act::Hurt, 16);
            self.vx = dir * 1.6;
            self.vz = 0.0;
            self.stun += 1;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// 小混混：普通出拳
    Thug,
    /// 刀手：快、打完就退開
    Knife,
    /// 大隻佬：慢、血厚、會衝撞
    Brute,
    /// 投擲手：保持距離丟東西
    Thrower,
    /// 大魔王（每一大關不同招式）
    Boss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    /// 小補血
    Apple,
    /// 中補血
    Meat,
    /// 全滿
    Feast,
    /// 分數
    Gem,
    /// 武器：鈍器（鐵管、木棍、十字鎬…每一大關樣子不同），一揮就把人打倒
    Bat,
    /// 武器：刀類（小刀、冰柱、短劍…），出手快
    Blade,
}

impl ItemKind {
    pub fn is_weapon(self) -> bool {
        matches!(self, ItemKind::Bat | ItemKind::Blade)
    }
    /// 武器能打中幾次
    fn uses(self) -> i32 {
        match self {
            ItemKind::Bat => 8,
            ItemKind::Blade => 10,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ShotKind {
    /// 雜兵丟的石頭（各關畫成不同東西）
    Rock,
    Bubble,
    Ice,
    Fire,
    Dark,
    /// 月華的月光波
    Moon,
    /// 玩家丟出去的武器
    Bat,
    Blade,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub slot: usize,
    pub character: String,
    #[serde(skip)]
    pub stats: Stats,
    #[serde(skip)]
    pub special: Special,
    #[serde(flatten)]
    pub body: Body,
    pub lives: i32,
    pub score: u32,
    /// 必殺氣（0..=SP_MAX）：普通攻擊打中、被打會集氣，滿了才能放必殺
    pub sp: i32,
    /// 手上的武器、還能打中幾次
    pub weapon: Option<ItemKind>,
    pub weapon_uses: i32,
    /// 用完命、出局（下一小關會以 1 條命回來）
    pub out: bool,
    /// 連段第幾下（0 起算）
    pub combo: u32,
    #[serde(skip)]
    chain_t: i32,
    #[serde(skip)]
    knees: u32,
    /// 連按兩下判定：上一格的左右、上一次按下的方向、還剩幾格可以按第二下
    #[serde(skip)]
    prev_dx: f32,
    #[serde(skip)]
    tap_dir: f32,
    #[serde(skip)]
    tap_timer: i32,
    /// 剛剛被誰抓、被誰打（給台詞用）
    #[serde(skip)]
    pub kos: u32,
}

#[derive(Clone, Debug, Default)]
struct Brain {
    cool: i32,
    retreat: i32,
    /// 繞圈時的前後偏移
    offset: f32,
    entered: bool,
    /// 魔王：下一個要用的招式
    plan: u8,
    summoned: u8,
    /// 魔王：對空上勾拳的冷卻
    antiair: i32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Enemy {
    pub id: u32,
    pub kind: Kind,
    pub elite: bool,
    #[serde(flatten)]
    pub body: Body,
    #[serde(skip)]
    brain: Brain,
    #[serde(skip)]
    speed: f32,
    #[serde(skip)]
    power: f32,
    #[serde(skip)]
    last_by: Option<usize>,
    /// 魔王的霸體（被打不會硬直）剩餘影格
    #[serde(skip)]
    armor: i32,
    #[serde(skip)]
    thrown_dmg: i32,
}

impl Enemy {
    /// 已經走進畫面、可以被打
    pub fn brain_entered(&self) -> bool {
        self.brain.entered
    }
    /// 身體半寬（被打的判定量到這裡）：照畫面上的身體寬度，菁英大一號
    fn body_r(&self) -> f32 {
        let r = match self.kind {
            Kind::Boss => 46.0,
            Kind::Brute => 34.0,
            _ => 24.0,
        };
        if self.elite { r * 1.25 } else { r }
    }
    fn reach(&self) -> f32 {
        match self.kind {
            Kind::Boss => 96.0,
            Kind::Brute => 80.0,
            Kind::Knife => 72.0,
            _ => 60.0,
        }
    }
    fn score(&self) -> u32 {
        let base = match self.kind {
            Kind::Thug => 100,
            Kind::Knife => 150,
            Kind::Thrower => 200,
            Kind::Brute => 300,
            Kind::Boss => 5000,
        };
        if self.elite {
            base * 5
        } else {
            base
        }
    }
    fn grabbable(&self) -> bool {
        self.kind != Kind::Boss && !(self.kind == Kind::Brute && self.elite)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Prop {
    pub id: u32,
    pub x: f32,
    pub z: f32,
    pub hp: i32,
    pub flash: i32,
    #[serde(skip)]
    drop: ItemKind,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: u32,
    pub kind: ItemKind,
    pub x: f32,
    pub z: f32,
    pub y: f32,
    #[serde(skip)]
    vy: f32,
    /// 武器剩下幾次（被打掉在地上的武器，撿回來接著用）
    #[serde(skip)]
    uses: i32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shot {
    pub id: u32,
    pub kind: ShotKind,
    pub x: f32,
    pub z: f32,
    pub y: f32,
    pub vx: f32,
    #[serde(skip)]
    vz: f32,
    #[serde(skip)]
    life: i32,
    #[serde(skip)]
    dmg: i32,
    #[serde(skip)]
    knock: bool,
    /// 玩家放的（月光波）：打敵人、會貫穿；None = 敵人丟的，打玩家
    pub owner: Option<usize>,
    #[serde(skip)]
    hits: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    /// 關卡標題（「1-1 白色都市」）
    Intro,
    Play,
    /// 小關突破
    Clear,
    /// 全軍覆沒（可以接關）
    Over,
    /// 六大關全部突破
    Ending,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Event {
    Stage { world: usize, sub: usize },
    Swing { slot: usize },
    Jump { slot: usize },
    Hit { x: f32, y: f32, big: bool, by: Option<usize> },
    Hurt { slot: usize, x: f32, y: f32, big: bool },
    Special { slot: usize },
    /// 按了必殺但氣不夠
    NoSp { slot: usize },
    Grab { slot: usize },
    Throw { slot: usize },
    /// 武器用完壞掉
    WeaponBreak { x: f32, z: f32 },
    Land { x: f32, z: f32 },
    /// 跑步的腳步（揚起灰塵）
    Step { x: f32, z: f32 },
    Ko { x: f32, z: f32, boss: bool, by: Option<usize> },
    Break { x: f32, z: f32 },
    Pickup { slot: usize, kind: ItemKind },
    Shoot { x: f32, y: f32 },
    Quake { x: f32, z: f32 },
    Wave,
    Go,
    Boss,
    Down { slot: usize },
    Respawn { slot: usize },
    Clear { world: usize, sub: usize },
    Over,
    Ending,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    pub world: usize,
    pub sub: usize,
    pub length: f32,
    pub camera: f32,
    /// 鏡頭鎖住中（打完這一波才放行）
    pub locked: bool,
    /// 顯示「GO →」
    pub go: bool,
    pub phase: Phase,
    pub timer: i32,
    pub difficulty: Difficulty,
    pub players: Vec<Player>,
    pub enemies: Vec<Enemy>,
    pub props: Vec<Prop>,
    pub items: Vec<Item>,
    pub shots: Vec<Shot>,
    /// 魔王的 id（畫面上方畫血條）
    pub boss: Option<u32>,
    pub continues: u32,
    pub frame: u64,
    pub events: Vec<Event>,
    #[serde(skip)]
    def: StageDef,
    #[serde(skip)]
    wave: usize,
    #[serde(skip)]
    wave_on: bool,
    #[serde(skip)]
    pending: Vec<(i32, Spawn)>,
    #[serde(skip)]
    next_id: u32,
    #[serde(skip)]
    rng: SmallRng,
    #[serde(skip)]
    start_lives: i32,
    #[serde(skip)]
    go_from: f32,
    /// 打擊停頓剩幾格；停頓中按的鍵先存起來，停完再用（不會吃掉連段的按鍵）
    #[serde(skip)]
    hitstop: i32,
    #[serde(skip)]
    buffered: Vec<Input>,
    /// 這一小關開始時每位玩家的（命、體力、分數、必殺氣）：暫停選單「從本關開頭」用
    #[serde(skip)]
    stage_start: Vec<(i32, i32, u32, i32)>,
}

/// 每位參加者：(座位 slot, 角色 id)
pub struct Entrant<'a> {
    pub slot: usize,
    pub character: &'a str,
}

impl Match {
    pub fn new(entrants: &[Entrant], world: usize, difficulty: Difficulty, lives: i32, seed: u64) -> Self {
        let players = entrants
            .iter()
            .take(MAX_PLAYERS)
            .enumerate()
            .map(|(i, e)| {
                let stats = stats_of(e.character);
                Player {
                    slot: e.slot,
                    character: e.character.to_string(),
                    stats,
                    special: special_of(e.character),
                    body: Body::new(120.0 + i as f32 * 40.0, Z_TOP + 40.0 + i as f32 * 36.0, stats.hp, 1.0),
                    lives: lives.max(1),
                    score: 0,
                    out: false,
                    combo: 0,
                    chain_t: 0,
                    knees: 0,
                    prev_dx: 0.0,
                    tap_dir: 0.0,
                    tap_timer: 0,
                    kos: 0,
                    sp: SP_START,
                    weapon: None,
                    weapon_uses: 0,
                }
            })
            .collect();
        let world = world.min(WORLDS - 1);
        let mut m = Match {
            world,
            sub: 0,
            length: 0.0,
            camera: 0.0,
            locked: false,
            go: false,
            phase: Phase::Intro,
            timer: INTRO_FRAMES,
            difficulty,
            players,
            enemies: Vec::new(),
            props: Vec::new(),
            items: Vec::new(),
            shots: Vec::new(),
            boss: None,
            continues: 0,
            frame: 0,
            events: Vec::new(),
            def: StageDef { length: W, waves: Vec::new(), props: Vec::new() },
            wave: 0,
            wave_on: false,
            pending: Vec::new(),
            next_id: 1,
            rng: SmallRng::seed_from_u64(seed),
            start_lives: lives.max(1),
            go_from: 0.0,
            hitstop: 0,
            buffered: Vec::new(),
            stage_start: Vec::new(),
        };
        m.load_stage(world, 0);
        m
    }

    /// 換到某一小關：清掉敵人道具、把玩家排到左邊
    fn load_stage(&mut self, world: usize, sub: usize) {
        self.world = world;
        self.sub = sub;
        self.def = build_stage(world, sub, self.players.len(), self.difficulty);
        self.length = self.def.length;
        self.camera = 0.0;
        self.locked = false;
        self.go = false;
        self.wave = 0;
        self.wave_on = false;
        self.pending.clear();
        self.enemies.clear();
        self.items.clear();
        self.shots.clear();
        self.boss = None;
        self.props = self
            .def
            .props
            .clone()
            .into_iter()
            .map(|(x, z, drop)| {
                let id = self.next_id;
                self.next_id += 1;
                Prop { id, x, z, hp: 2, flash: 0, drop }
            })
            .collect();
        let n = self.players.len();
        for (i, p) in self.players.iter_mut().enumerate() {
            if p.out {
                p.out = false;
                p.lives = 1;
                p.body.hp = p.body.max_hp;
            }
            let b = &mut p.body;
            b.x = 100.0 + i as f32 * 46.0;
            b.z = Z_TOP + 30.0 + (Z_BOTTOM - Z_TOP - 60.0) * if n > 1 { i as f32 / (n - 1) as f32 } else { 0.5 };
            b.y = 0.0;
            b.vx = 0.0;
            b.vy = 0.0;
            b.vz = 0.0;
            b.facing = 1.0;
            b.hold = None;
            b.inv = 0;
            if b.hp <= 0 {
                b.hp = b.max_hp;
            }
            b.set(Act::Stand, 0);
        }
        self.phase = Phase::Intro;
        self.timer = INTRO_FRAMES;
        self.events.push(Event::Stage { world, sub });
        self.stage_start = self.players.iter().map(|p| (p.lives, p.body.hp, p.score, p.sp)).collect();
    }

    /// 從這一小關的開頭重來（暫停選單）：每位玩家回到進這一關時的命、體力、分數、必殺氣
    pub fn restart_stage(&mut self) {
        let start = std::mem::take(&mut self.stage_start);
        for (p, &(lives, hp, score, sp)) in self.players.iter_mut().zip(&start) {
            p.out = false;
            p.lives = lives;
            p.body.hp = hp;
            p.score = score;
            p.sp = sp;
        }
        self.hitstop = 0;
        self.buffered.clear();
        let (w, s) = (self.world, self.sub);
        self.load_stage(w, s);
    }

    /// 打擊停頓中按下的鍵先記著
    fn buffer(&mut self, inputs: &[Input]) {
        self.buffered.resize(inputs.len(), Input::default());
        for (b, i) in self.buffered.iter_mut().zip(inputs) {
            b.attack_pressed |= i.attack_pressed;
            b.jump_pressed |= i.jump_pressed;
            b.special_pressed |= i.special_pressed;
        }
    }

    /// 停頓結束：把記著的按鍵併進這一格
    fn take_buffered(&mut self, inputs: &[Input]) -> Vec<Input> {
        let mut out = inputs.to_vec();
        for (o, b) in out.iter_mut().zip(self.buffered.drain(..)) {
            o.attack_pressed |= b.attack_pressed;
            o.jump_pressed |= b.jump_pressed;
            o.special_pressed |= b.special_pressed;
        }
        out
    }

    /// 全軍覆沒後接關：從這一小關重來，每人命數回滿
    pub fn continue_game(&mut self) -> bool {
        if self.phase != Phase::Over {
            return false;
        }
        self.continues += 1;
        for p in &mut self.players {
            p.out = false;
            p.lives = self.start_lives;
            p.body.hp = p.body.max_hp;
            p.score = p.score / 2;
        }
        let (w, s) = (self.world, self.sub);
        self.load_stage(w, s);
        true
    }

    /// 被打倒的玩家手上的武器掉在地上（剩下的次數留著）；短刀手被打倒有一半機會掉出他的刀
    fn drop_weapons(&mut self) {
        let mut drops = Vec::new();
        for p in self.players.iter_mut() {
            if p.body.act == Act::Fall && p.body.t == 0 {
                if let Some(kind) = p.weapon.take() {
                    drops.push((p.body.x, p.body.z, kind, p.weapon_uses));
                }
            }
        }
        for e in self.enemies.iter() {
            if e.kind == Kind::Knife && e.body.act == Act::Dead && e.body.t == 1 && e.body.x as i32 % 2 == 0 {
                drops.push((e.body.x, e.body.z, ItemKind::Blade, ItemKind::Blade.uses()));
            }
        }
        for (x, z, kind, uses) in drops {
            let id = self.new_id();
            self.items.push(Item { id, kind, x, z, y: 30.0, vy: 4.0, uses });
        }
    }

    /// 在地上放一個道具（測試、之後的關卡機關用）
    pub fn spawn_item(&mut self, kind: ItemKind, x: f32, z: f32) {
        let id = self.new_id();
        self.items.push(Item { id, kind, x, z, y: 0.0, vy: 0.0, uses: kind.uses() });
    }

    fn new_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    /// 現在還能動的玩家
    pub fn active_players(&self) -> impl Iterator<Item = &Player> {
        self.players.iter().filter(|p| !p.out && p.body.alive())
    }

    pub fn step(&mut self, inputs: &[Input]) {
        self.events.clear();
        self.frame += 1;
        match self.phase {
            Phase::Intro => {
                self.timer -= 1;
                self.update_players(inputs, false);
                if self.timer <= 0 {
                    self.phase = Phase::Play;
                }
            }
            Phase::Play if self.hitstop > 0 => {
                self.hitstop -= 1;
                self.buffer(inputs);
            }
            Phase::Play => {
                let inputs = &self.take_buffered(inputs);
                self.update_players(inputs, true);
                self.update_enemies();
                self.resolve_player_attacks();
                self.resolve_enemy_attacks();
                self.update_shots();
                self.drop_weapons();
                self.update_items();
                self.spawn_pending();
                self.update_camera();
                self.update_waves();
                self.check_over();
                // 被打也會集氣（被圍著打的時候比較快能放必殺脫身）
                for e in &self.events {
                    if let Event::Hurt { slot, .. } = *e {
                        if let Some(p) = self.players.iter_mut().find(|p| p.slot == slot) {
                            gain_sp(p, SP_HURT);
                        }
                    }
                }
            }
            Phase::Clear => {
                self.timer -= 1;
                for p in &mut self.players {
                    p.body.t += 1;
                    p.body.air();
                }
                if self.timer <= 0 {
                    if self.world + 1 >= WORLDS && self.sub + 1 >= SUBS {
                        self.phase = Phase::Ending;
                        self.timer = 0;
                        self.events.push(Event::Ending);
                    } else {
                        // 下一關之前回一半的血
                        for p in &mut self.players {
                            p.body.hp = (p.body.hp + (p.body.max_hp - p.body.hp + 1) / 2).min(p.body.max_hp);
                        }
                        let (w, s) = if self.sub + 1 < SUBS { (self.world, self.sub + 1) } else { (self.world + 1, 0) };
                        self.load_stage(w, s);
                    }
                }
            }
            Phase::Over | Phase::Ending => {
                self.timer += 1;
                for p in &mut self.players {
                    p.body.t += 1;
                }
                for e in &mut self.enemies {
                    e.body.t += 1;
                }
            }
        }
    }

    // ---------- 玩家 ----------

    fn update_players(&mut self, inputs: &[Input], can_fight: bool) {
        let (left, right) = (self.camera + 24.0, self.camera + W - 24.0);
        // 地上的武器（站在旁邊按攻擊就撿起來）
        let weapons: Vec<(u32, f32, f32, ItemKind, i32)> = self
            .items
            .iter()
            .filter(|it| it.kind.is_weapon() && it.y <= 0.0)
            .map(|it| (it.id, it.x, it.z, it.kind, it.uses))
            .collect();
        let mut picked: Vec<u32> = Vec::new();
        // 打得到的敵人位置（出拳時自動上下對準用）
        let foes: Vec<(f32, f32)> = self
            .enemies
            .iter()
            .filter(|e| e.brain.entered && e.body.vulnerable() && e.body.act != Act::Held && e.body.y <= 0.0)
            .map(|e| (e.body.x, e.body.z))
            .collect();
        for i in 0..self.players.len() {
            let inp = if can_fight { inputs.get(i).copied().unwrap_or_default() } else {
                // 關卡標題中：只能走路
                let mut x = inputs.get(i).copied().unwrap_or_default();
                x.attack_pressed = false;
                x.jump_pressed = false;
                x.special_pressed = false;
                x
            };
            let p = &mut self.players[i];
            if p.out {
                continue;
            }
            let slot = p.slot;
            let speed = p.stats.speed;
            let sp = p.special;
            let power = p.stats.power;
            let ready = p.sp >= SP_MAX;
            if inp.special_pressed && !ready && matches!(p.body.act, Act::Stand | Act::Walk | Act::Run) {
                self.events.push(Event::NoSp { slot });
            }
            let b = &mut p.body;
            b.t += 1;
            b.inv = (b.inv - 1).max(0);
            b.flash = (b.flash - 1).max(0);
            p.chain_t = (p.chain_t - 1).max(0);
            // 連按兩下左右：放開後很快又按同一個方向
            let dx_now = inp.dx();
            let mut double_tap = false;
            p.tap_timer = (p.tap_timer - 1).max(0);
            if dx_now != 0.0 && p.prev_dx == 0.0 {
                if p.tap_timer > 0 && p.tap_dir == dx_now {
                    double_tap = true;
                    p.tap_timer = 0;
                } else {
                    p.tap_dir = dx_now;
                    p.tap_timer = DOUBLE_TAP;
                }
            }
            p.prev_dx = dx_now;
            match b.act {
                Act::Run => {
                    // 放開方向、或按了反方向：停下來
                    if ready && (inp.special_pressed || (inp.attack_pressed && inp.jump_pressed)) {
                        begin_special(b, &sp);
                        p.sp = 0;
                        self.events.push(Event::Special { slot });
                    } else if inp.jump_pressed {
                        // 跑步跳：跳得比較遠
                        b.set(Act::Jump, 0);
                        b.vy = JUMP_V;
                        b.vx = b.facing * speed * RUN_SPEED;
                        b.vz = inp.dz() * speed * Z_SPEED;
                        self.events.push(Event::Jump { slot });
                    } else if inp.attack_pressed && p.weapon.is_some() {
                        // 跑步中按攻擊：把武器丟出去（飛得遠、穿過一整排）
                        let kind = p.weapon.take().unwrap_or(ItemKind::Bat);
                        self.next_id += 1;
                        self.shots.push(Shot {
                            id: self.next_id,
                            kind: if kind == ItemKind::Bat { ShotKind::Bat } else { ShotKind::Blade },
                            x: b.x + b.facing * 40.0,
                            z: b.z,
                            y: 70.0,
                            vx: b.facing * 11.0,
                            vz: 0.0,
                            life: 70,
                            dmg: (18.0 * power) as i32,
                            knock: true,
                            owner: Some(slot),
                            hits: Vec::new(),
                        });
                        b.attack(Move::Throw);
                        self.events.push(Event::Throw { slot });
                    } else if inp.attack_pressed {
                        b.attack(Move::DashAttack);
                        b.vx = b.facing * speed * 1.7;
                        self.events.push(Event::Swing { slot });
                    } else if inp.dx() != b.facing {
                        b.set(if inp.dx() != 0.0 || inp.dz() != 0.0 { Act::Walk } else { Act::Stand }, 0);
                    } else {
                        b.x += b.facing * speed * RUN_SPEED;
                        b.z += inp.dz() * speed * Z_SPEED;
                        if b.t % 10 == 0 {
                            self.events.push(Event::Step { x: b.x, z: b.z });
                        }
                    }
                }
                Act::Stand | Act::Walk if double_tap => {
                    b.facing = dx_now;
                    b.set(Act::Run, 0);
                    self.events.push(Event::Step { x: b.x, z: b.z });
                }
                Act::Stand | Act::Walk => {
                    if ready && (inp.special_pressed || (inp.attack_pressed && inp.jump_pressed)) {
                        begin_special(b, &sp);
                        p.sp = 0;
                        self.events.push(Event::Special { slot });
                    } else if inp.jump_pressed {
                        b.set(Act::Jump, 0);
                        b.vy = JUMP_V;
                        b.vx = inp.dx() * speed * 1.15;
                        b.vz = inp.dz() * speed * Z_SPEED;
                        if inp.dx() != 0.0 {
                            b.facing = inp.dx();
                        }
                        self.events.push(Event::Jump { slot });
                    } else if let Some(&(id, _, _, kind, uses)) = (inp.attack_pressed && p.weapon.is_none())
                        .then(|| weapons.iter().find(|w| !picked.contains(&w.0) && (w.1 - b.x).abs() < 40.0 && (w.2 - b.z).abs() < 20.0))
                        .flatten()
                    {
                        // 撿武器：彎腰一下
                        picked.push(id);
                        p.weapon = Some(kind);
                        p.weapon_uses = uses;
                        b.attack(Move::None);
                        b.dur = 8;
                        self.events.push(Event::Pickup { slot, kind });
                    } else if inp.attack_pressed && p.weapon.is_some() {
                        // 拿著武器：用武器打
                        b.attack(if p.weapon == Some(ItemKind::Bat) { Move::Bat } else { Move::Blade });
                        if inp.dx() != 0.0 {
                            b.facing = inp.dx();
                        }
                        align_z(b, &foes);
                        self.events.push(Event::Swing { slot });
                    } else if inp.attack_pressed {
                        // 上一下有打中，而且接得夠快：接下一段
                        let next = if p.chain_t > 0 && b.connected { (p.combo + 1) % 3 } else { 0 };
                        p.combo = next;
                        b.attack([Move::Jab, Move::Jab2, Move::Kick][next as usize]);
                        if inp.dx() != 0.0 {
                            b.facing = inp.dx();
                        }
                        align_z(b, &foes);
                        self.events.push(Event::Swing { slot });
                    } else {
                        let (dx, dz) = (inp.dx(), inp.dz());
                        let moving = dx != 0.0 || dz != 0.0;
                        let diag = if dx != 0.0 && dz != 0.0 { 0.8 } else { 1.0 };
                        b.x += dx * speed * diag;
                        b.z += dz * speed * Z_SPEED * diag;
                        if dx != 0.0 {
                            b.facing = dx;
                        }
                        if moving && b.act != Act::Walk {
                            b.set(Act::Walk, 0);
                        } else if !moving && b.act != Act::Stand {
                            b.set(Act::Stand, 0);
                        }
                        b.stun = 0;
                        // 走向正在硬直的敵人：抓住
                        if dx != 0.0 {
                            let (bx, bz, facing) = (b.x, b.z, b.facing);
                            if let Some(e) = self.enemies.iter_mut().find(|e| {
                                e.grabbable()
                                    && e.body.act == Act::Hurt
                                    && (e.body.z - bz).abs() < 12.0
                                    && (e.body.x - bx) * facing > 0.0
                                    && (e.body.x - bx).abs() < 46.0
                            }) {
                                e.body.set(Act::Held, 0);
                                e.body.hold = Some(slot as u32);
                                e.body.facing = -facing;
                                b.set(Act::Grab, 0);
                                b.hold = Some(e.id);
                                p.knees = 0;
                                self.events.push(Event::Grab { slot });
                            }
                        }
                    }
                }
                Act::Attack if b.mv == Move::Special => {
                    let live = b.t < sp.startup + sp.active;
                    if live && b.t >= sp.startup.min(2) {
                        b.x += b.facing * sp.dash;
                        b.z += inp.dz() * speed * Z_SPEED * 0.5;
                    }
                    if b.y > 0.0 || b.vy > 0.0 {
                        b.y += b.vy;
                        b.vy -= G;
                        if b.y <= 0.0 {
                            b.y = 0.0;
                            b.vy = 0.0;
                            self.events.push(Event::Land { x: b.x, z: b.z });
                        }
                    }
                    if b.t == sp.startup {
                        if sp.wave {
                            // 月光波：往前飛、貫穿，打中的人都會倒
                            self.next_id += 1;
                            self.shots.push(Shot {
                                id: self.next_id,
                                kind: ShotKind::Moon,
                                x: b.x + b.facing * 50.0,
                                z: b.z,
                                y: 60.0,
                                vx: b.facing * 9.0,
                                vz: 0.0,
                                life: 70,
                                dmg: (sp.dmg * power) as i32,
                                knock: true,
                                owner: Some(slot),
                                hits: Vec::new(),
                            });
                        }
                        if sp.quake {
                            self.events.push(Event::Quake { x: b.x, z: b.z });
                        }
                    }
                    if b.t >= b.dur && b.y <= 0.0 {
                        b.set(Act::Stand, 0);
                    }
                }
                Act::Attack => {
                    b.x += b.vx;
                    b.vx *= 0.8;
                    // 衝刺攻擊打空：收招多 14 格（不能一直衝來衝去）
                    if b.mv == Move::DashAttack && b.t == b.dur - 1 && !b.connected && b.dur == Move::DashAttack.total() {
                        b.dur += 14;
                    }
                    if b.mv == Move::Kick && b.t < 8 {
                        b.x += b.facing * 1.6;
                    }
                    if b.t >= b.dur {
                        if b.connected {
                            p.chain_t = 26;
                        }
                        let keep = b.connected;
                        b.set(Act::Stand, 0);
                        b.connected = keep;
                    }
                }
                Act::Jump => {
                    if inp.attack_pressed && b.mv != Move::JumpKick {
                        b.mv = Move::JumpKick;
                        b.hit_list.clear();
                        self.events.push(Event::Swing { slot });
                    }
                    if b.air() {
                        self.events.push(Event::Land { x: b.x, z: b.z });
                        b.attack(Move::None);
                        // 跳踢落地的硬直比較長，不能一直跳踢
                        b.dur = if b.mv == Move::JumpKick { 14 } else { 7 };
                    }
                }
                Act::Hurt => {
                    b.x += b.vx;
                    b.vx *= 0.85;
                    if b.t >= b.dur {
                        b.set(Act::Stand, 0);
                    }
                }
                Act::Fall => {
                    if b.air() {
                        self.events.push(Event::Land { x: b.x, z: b.z });
                        if b.hp <= 0 {
                            b.set(Act::Dead, 0);
                            self.events.push(Event::Down { slot });
                        } else {
                            b.set(Act::Down, 34);
                        }
                    }
                }
                Act::Down => {
                    if b.t >= b.dur {
                        b.set(Act::Rise, 16);
                        b.inv = 60;
                    }
                }
                Act::Rise => {
                    if b.t >= b.dur {
                        b.set(Act::Stand, 0);
                    }
                }
                Act::Grab => {
                    let id = b.hold;
                    let held = self.enemies.iter_mut().find(|e| Some(e.id) == id && e.body.act == Act::Held);
                    let Some(e) = held else {
                        b.hold = None;
                        b.set(Act::Stand, 0);
                        continue;
                    };
                    e.body.x = b.x + b.facing * 40.0;
                    e.body.z = b.z + 1.0;
                    e.body.facing = -b.facing;
                    e.body.t = 0;
                    if b.mv == Move::Knee && b.t >= b.dur {
                        b.mv = Move::None;
                    }
                    let back = inp.dx() != 0.0 && inp.dx() == -b.facing;
                    if inp.attack_pressed && b.mv == Move::None {
                        if back || p.knees >= 2 {
                            // 丟出去：往後丟就先轉身
                            if back {
                                b.facing = -b.facing;
                                e.body.x = b.x + b.facing * 40.0;
                            }
                            let dmg = (16.0 * p.stats.power) as i32;
                            e.body.set(Act::Thrown, 0);
                            e.body.hold = None;
                            e.body.vx = b.facing * 8.0;
                            e.body.vy = 6.5;
                            e.body.y = 8.0;
                            e.thrown_dmg = dmg;
                            e.last_by = Some(slot);
                            b.hold = None;
                            b.attack(Move::Throw);
                            p.score += dmg as u32 * 10;
                            p.sp = (p.sp + SP_BIG).min(SP_MAX);
                            self.events.push(Event::Throw { slot });
                        } else {
                            p.knees += 1;
                            let dmg = (move_data(Move::Knee).dmg * p.stats.power) as i32;
                            e.body.hp = (e.body.hp - dmg).max(1);
                            e.body.flash = 7;
                            e.last_by = Some(slot);
                            b.mv = Move::Knee;
                            b.dur = b.t + 10;
                            p.score += dmg as u32 * 10;
                            p.sp = (p.sp + SP_HIT).min(SP_MAX);
                            self.events.push(Event::Hit { x: e.body.x, y: e.body.z - 70.0, big: false, by: Some(slot) });
                        }
                    } else if b.t > 110 {
                        // 抓太久，敵人掙脫
                        e.body.set(Act::Stand, 0);
                        e.body.hold = None;
                        e.body.x += -b.facing * -8.0;
                        b.hold = None;
                        b.set(Act::Stand, 0);
                    }
                }
                Act::Dead => {
                    if b.t >= RESPAWN_FRAMES {
                        p.lives -= 1;
                        if p.lives > 0 {
                            // 從畫面上方掉回來
                            b.hp = b.max_hp;
                            p.sp = p.sp.max(SP_START);
                            b.x = (self.camera + 160.0).max(b.x.min(self.camera + W - 160.0));
                            b.z = (Z_TOP + Z_BOTTOM) / 2.0;
                            b.y = 420.0;
                            b.vy = 0.0;
                            b.vx = 0.0;
                            b.vz = 0.0;
                            b.set(Act::Enter, 0);
                            b.inv = 150;
                            self.events.push(Event::Respawn { slot });
                        } else {
                            p.out = true;
                        }
                    }
                }
                Act::Enter => {
                    b.vy -= G * 0.6;
                    b.y += b.vy;
                    if b.y <= 0.0 {
                        b.y = 0.0;
                        b.vy = 0.0;
                        b.set(Act::Stand, 0);
                        self.events.push(Event::Land { x: b.x, z: b.z });
                    }
                }
                Act::Win => {}
                _ => {
                    b.set(Act::Stand, 0);
                }
            }
            let b = &mut self.players[i].body;
            b.x = b.x.clamp(left, right);
            b.z = b.z.clamp(Z_TOP, Z_BOTTOM);
        }
        self.items.retain(|it| !picked.contains(&it.id));
    }

    // ---------- 敵人 ----------

    fn update_enemies(&mut self) {
        // 正在跳的玩家（魔王對空用）
        let jumpers: Vec<(f32, f32, bool)> = self
            .players
            .iter()
            .filter(|p| !p.out && p.body.act == Act::Jump)
            .map(|p| (p.body.x, p.body.z, true))
            .collect();
        let targets: Vec<(f32, f32, bool)> = self
            .players
            .iter()
            .map(|p| (p.body.x, p.body.z, !p.out && p.body.alive() && p.body.act != Act::Enter))
            .collect();
        // 每個玩家身邊最近的幾隻敵人負責進攻，其他的在旁邊繞
        let mut engaged = vec![false; self.enemies.len()];
        let mut target_of = vec![None; self.enemies.len()];
        for (i, e) in self.enemies.iter().enumerate() {
            let mut best: Option<(usize, f32)> = None;
            for (k, t) in targets.iter().enumerate() {
                if !t.2 {
                    continue;
                }
                let d = (t.0 - e.body.x).abs() + (t.1 - e.body.z).abs() * 2.0;
                if best.is_none_or(|b| d < b.1) {
                    best = Some((k, d));
                }
            }
            target_of[i] = best.map(|b| b.0);
        }
        for k in 0..targets.len() {
            let mut close: Vec<(usize, f32)> = self
                .enemies
                .iter()
                .enumerate()
                .filter(|(i, e)| target_of[*i] == Some(k) && e.kind != Kind::Thrower)
                .map(|(i, e)| (i, (e.body.x - targets[k].0).abs()))
                .collect();
            close.sort_by(|a, b| a.1.total_cmp(&b.1));
            for (rank, (i, _)) in close.iter().enumerate() {
                if rank < ATTACKERS_PER_PLAYER || self.enemies[*i].kind == Kind::Boss {
                    engaged[*i] = true;
                }
            }
        }

        // 正在出手的敵人：每位玩家的左右兩邊各最多一隻在打他（雙截龍式的輪流上，不會一群一起打；
        // 被前後包夾才會兩邊一起來，所以站位很重要）。簡單只有一隻
        let per_side = self.difficulty != Difficulty::Easy;
        let side_of = |ex: f32, k: usize| usize::from(ex > targets[k].0);
        let mut swinging = vec![[0usize; 2]; targets.len()];
        for (i, e) in self.enemies.iter().enumerate() {
            if let Some(k) = target_of[i] {
                if e.body.act == Act::Attack && e.body.mv != Move::None && e.kind != Kind::Thrower {
                    swinging[k][side_of(e.body.x, k)] += 1;
                }
            }
        }
        let cam = self.camera;
        let (cool_lo, cool_hi) = match self.difficulty {
            Difficulty::Easy => (70, 140),
            Difficulty::Normal => (40, 100),
            Difficulty::Hard => (20, 65),
        };
        let world = self.world;
        let mut new_shots = Vec::new();
        let mut summons = 0;
        for i in 0..self.enemies.len() {
            let target = target_of[i].map(|k| targets[k]);
            let target_k = target_of[i];
            let e = &mut self.enemies[i];
            let reach = e.reach();
            let b = &mut e.body;
            b.t += 1;
            b.flash = (b.flash - 1).max(0);
            b.inv = (b.inv - 1).max(0);
            e.armor = (e.armor - 1).max(0);
            match b.act {
                Act::Stand | Act::Walk => {
                    e.brain.cool -= 1;
                    e.brain.retreat = (e.brain.retreat - 1).max(0);
                    if !e.brain.entered {
                        // 先走進畫面
                        let dir = if b.x < cam + W / 2.0 { 1.0 } else { -1.0 };
                        b.x += dir * e.speed.max(2.0);
                        b.facing = dir;
                        if b.act != Act::Walk {
                            b.set(Act::Walk, 0);
                        }
                        if b.x > cam + 30.0 && b.x < cam + W - 30.0 {
                            e.brain.entered = true;
                        }
                        continue;
                    }
                    let Some((tx, tz, _)) = target else {
                        if b.act != Act::Stand {
                            b.set(Act::Stand, 0);
                        }
                        continue;
                    };
                    let side = if b.x < tx { -1.0 } else { 1.0 };
                    let dx = tx - b.x;
                    let dz = tz - b.z;
                    // 魔王：有人在身邊跳起來（想跳踢）就用上勾拳打下來
                    e.brain.antiair = (e.brain.antiair - 1).max(0);
                    if e.kind == Kind::Boss && e.brain.antiair == 0 {
                        if let Some(&(jx, _, _)) = jumpers.iter().find(|(jx, jz, _)| (jx - b.x).abs() < 150.0 && (jz - b.z).abs() < 30.0) {
                            b.facing = (jx - b.x).signum();
                            b.attack(Move::Uppercut);
                            e.brain.antiair = 80;
                            continue;
                        }
                    }
                    let r: f32 = self.rng.gen();
                    // 魔王的特殊招式
                    if e.kind == Kind::Boss && e.brain.cool <= 0 {
                        let hp_ratio = b.hp as f32 / b.max_hp as f32;
                        if world == 5 && ((e.brain.summoned == 0 && hp_ratio < 0.66) || (e.brain.summoned == 1 && hp_ratio < 0.33)) {
                            e.brain.summoned += 1;
                            summons += 2;
                            b.set(Act::Cast, 36);
                            e.brain.plan = 9;
                            e.brain.cool = 60;
                            continue;
                        }
                        let kit = boss_kit(world);
                        let far = dx.abs() > reach + 30.0;
                        let pick = (e.brain.plan as usize + (r * 10.0) as usize) % 4;
                        e.brain.plan = e.brain.plan.wrapping_add(1);
                        let choice = match pick {
                            0 if kit.charge && far && dz.abs() < 40.0 => Some(Act::Charge),
                            1 if kit.leap => Some(Act::Leap),
                            2 if kit.shots > 0 && far => Some(Act::Cast),
                            3 if kit.teleport => Some(Act::Vanish),
                            _ => None,
                        };
                        if let Some(act) = choice {
                            b.facing = if dx >= 0.0 { 1.0 } else { -1.0 };
                            match act {
                                Act::Charge => {
                                    b.set(Act::Charge, 70);
                                    b.z += dz.clamp(-30.0, 30.0);
                                }
                                Act::Leap => {
                                    b.set(Act::Leap, 0);
                                    b.vy = 12.0;
                                    // 大約 40 格後落在玩家身上
                                    b.vx = (dx / 40.0).clamp(-9.0, 9.0);
                                    b.vz = (dz / 40.0).clamp(-3.0, 3.0);
                                    b.y = 1.0;
                                }
                                Act::Cast => {
                                    b.set(Act::Cast, 40);
                                    e.brain.plan = 0;
                                }
                                Act::Vanish => {
                                    b.set(Act::Vanish, 40);
                                    b.inv = 40;
                                }
                                _ => {}
                            }
                            e.brain.cool = (r * 60.0) as i32 + cool_lo + 40;
                            continue;
                        }
                    }
                    let (want_x, want_z);
                    if e.kind == Kind::Thrower {
                        want_x = tx + side * 230.0;
                        want_z = tz;
                        if dz.abs() < 14.0 && dx.abs() > 140.0 && dx.abs() < 420.0 && e.brain.cool <= 0 {
                            b.facing = dx.signum();
                            b.attack(Move::Shoot);
                            b.dur = 36;
                            e.brain.cool = cool_hi + 30;
                            continue;
                        }
                    } else if e.brain.retreat > 0 {
                        want_x = tx + side * 190.0;
                        want_z = tz + e.brain.offset;
                    } else if engaged[i] {
                        want_x = tx + side * reach * 0.75;
                        want_z = tz;
                    } else {
                        want_x = tx + side * 200.0;
                        want_z = tz + e.brain.offset;
                    }
                    // 進攻
                    let in_range = dx.abs() <= reach && dz.abs() <= 12.0;
                    let turn = e.kind == Kind::Boss
                        || e.kind == Kind::Thrower
                        || target_k.is_some_and(|k| {
                            let n = swinging[k];
                            if per_side { n[side_of(b.x, k)] == 0 } else { n[0] + n[1] == 0 }
                        });
                    if (engaged[i] || e.kind == Kind::Thrower) && in_range && e.brain.cool <= 0 && turn {
                        if let Some(k) = target_k {
                            swinging[k][side_of(b.x, k)] += 1;
                        }
                        b.facing = dx.signum();
                        let mv = match e.kind {
                            Kind::Thug | Kind::Thrower => Move::Punch,
                            Kind::Knife => Move::Slash,
                            Kind::Brute => Move::Slam,
                            Kind::Boss => Move::BossSlam,
                        };
                        b.attack(mv);
                        e.brain.cool = cool_lo + ((cool_hi - cool_lo) as f32 * r) as i32;
                        if e.kind == Kind::Knife {
                            e.brain.retreat = 50;
                        }
                        continue;
                    }
                    // 大隻佬：離遠了就衝過來
                    if e.kind == Kind::Brute && e.brain.cool <= 0 && dz.abs() < 18.0 && dx.abs() > 180.0 && dx.abs() < 420.0 && r < 0.02 {
                        b.facing = dx.signum();
                        b.set(Act::Charge, 60);
                        e.brain.cool = cool_hi;
                        continue;
                    }
                    let mx = want_x - b.x;
                    let mz = want_z - b.z;
                    let sp = e.speed;
                    let moving = mx.abs() > 4.0 || mz.abs() > 3.0;
                    if mx.abs() > 4.0 {
                        b.x += mx.signum() * sp.min(mx.abs());
                    }
                    if mz.abs() > 3.0 {
                        b.z += mz.signum() * (sp * Z_SPEED).min(mz.abs());
                    }
                    b.facing = if dx.abs() > 2.0 { dx.signum() } else { b.facing };
                    if moving && b.act != Act::Walk {
                        b.set(Act::Walk, 0);
                    } else if !moving && b.act != Act::Stand {
                        b.set(Act::Stand, 0);
                    }
                    b.stun = 0;
                }
                Act::Attack => {
                    if b.mv == Move::Shoot && b.t == 16 {
                        let kind = if e.kind == Kind::Boss { boss_kit(world).shot } else { ShotKind::Rock };
                        new_shots.push((b.x + b.facing * 30.0, b.z, 70.0, b.facing * 6.5, 0.0, kind, (9.0 * e.power) as i32, false));
                    }
                    if b.mv == Move::Slam && b.t < 10 {
                        b.x += b.facing * 0.6;
                    }
                    if b.t >= b.dur {
                        b.set(Act::Stand, 0);
                    }
                }
                Act::Charge => {
                    b.x += b.facing * if e.kind == Kind::Boss { 7.5 } else { 6.0 };
                    if b.t >= b.dur || b.x < cam - 40.0 || b.x > cam + W + 40.0 {
                        b.x = b.x.clamp(cam + 20.0, cam + W - 20.0);
                        b.set(Act::Stand, 0);
                    }
                }
                Act::Leap => {
                    if b.air() {
                        b.set(Act::Attack, 20);
                        b.mv = Move::None;
                        self.events.push(Event::Quake { x: b.x, z: b.z });
                        // 落地的震波：在地上的玩家都會被震倒
                        let (bx, bz) = (b.x, b.z);
                        let dmg = (16.0 * e.power) as i32;
                        for p in self.players.iter_mut() {
                            let pb = &mut p.body;
                            if !p.out && pb.vulnerable() && pb.y <= 0.0 && (pb.x - bx).abs() < 130.0 && (pb.z - bz).abs() < 40.0 {
                                let dir = if pb.x >= bx { 1.0 } else { -1.0 };
                                pb.take_hit(dmg, true, dir, false);
                                self.events.push(Event::Hurt { slot: p.slot, x: pb.x, y: pb.z - 60.0, big: true });
                            }
                        }
                    }
                }
                Act::Cast => {
                    if e.brain.plan == 9 {
                        // 召喚中（實際生出來在迴圈外）
                    } else if e.kind == Kind::Boss && b.t == 18 {
                        let kit = boss_kit(world);
                        let n = kit.shots as i32;
                        for k in 0..n {
                            let spread = if n > 1 { (k as f32 / (n - 1) as f32 - 0.5) * 2.0 } else { 0.0 };
                            new_shots.push((b.x + b.facing * 40.0, b.z, 90.0, b.facing * 6.0, spread * 1.6, kit.shot, (11.0 * e.power) as i32, kit.shot == ShotKind::Fire));
                        }
                    }
                    if b.t >= b.dur {
                        e.brain.plan = 0;
                        b.set(Act::Stand, 0);
                    }
                }
                Act::Vanish => {
                    if b.t == 20 {
                        // 移到玩家背後
                        if let Some((tx, tz, _)) = target {
                            let behind = if b.x < tx { 1.0 } else { -1.0 };
                            b.x = (tx + behind * 80.0).clamp(cam + 40.0, cam + W - 40.0);
                            b.z = tz;
                            b.facing = -behind;
                        }
                    }
                    if b.t >= b.dur {
                        b.attack(Move::BossSlam);
                    }
                }
                Act::Hurt => {
                    b.x += b.vx;
                    b.vx *= 0.85;
                    if b.t >= b.dur {
                        b.set(Act::Stand, 0);
                    }
                }
                Act::Held => {
                    if b.hold.is_none() {
                        b.set(Act::Stand, 0);
                    }
                }
                Act::Fall | Act::Thrown => {
                    let thrown = b.act == Act::Thrown;
                    if b.air() {
                        self.events.push(Event::Land { x: b.x, z: b.z });
                        if thrown {
                            b.hp = (b.hp - e.thrown_dmg).max(0);
                            e.thrown_dmg = 0;
                        }
                        if b.hp <= 0 {
                            b.set(Act::Dead, 0);
                        } else {
                            b.set(Act::Down, if e.kind == Kind::Boss { 26 } else { 40 });
                        }
                    }
                }
                Act::Down => {
                    if b.t >= b.dur {
                        b.set(Act::Rise, 16);
                        b.inv = 24;
                    }
                }
                Act::Rise => {
                    if b.t >= b.dur {
                        b.set(Act::Stand, 0);
                        if e.kind == Kind::Boss {
                            e.armor = 110;
                        }
                    }
                }
                _ => {}
            }
            let b = &mut e.body;
            b.z = b.z.clamp(Z_TOP, Z_BOTTOM);
            // 進場後就留在畫面裡（投擲手會想拉開距離，不能讓它躲到畫面外）
            // 倒地、死掉的只擋右邊（被打飛到右邊還是在畫面裡）、不擋左邊：鏡頭只會往右走，
            // 擋左邊的話鏡頭前進時倒地的人會被拖著走
            if e.brain.entered && matches!(b.act, Act::Down | Act::Dead) {
                b.x = b.x.min(cam + W - 24.0);
            }
            // 站著的貼著邊緣就擋住；離畫面比較遠的（倒地時被鏡頭甩在後面、站起來的）走回來，不會瞬間移動
            if e.brain.entered && matches!(b.act, Act::Stand | Act::Walk | Act::Attack | Act::Hurt | Act::Rise) {
                let (lo, hi) = (cam + 24.0, cam + W - 24.0);
                if b.x < lo {
                    b.x = if lo - b.x < 16.0 { lo } else { b.x + 6.0 };
                } else if b.x > hi {
                    b.x = if b.x - hi < 16.0 { hi } else { b.x - 6.0 };
                }
            }
        }
        // 被丟出去的敵人撞到其他敵人
        let flying: Vec<(u32, f32, f32, f32, Option<usize>)> = self
            .enemies
            .iter()
            .filter(|e| e.body.act == Act::Thrown)
            .map(|e| (e.id, e.body.x, e.body.z, e.body.vx.signum(), e.last_by))
            .collect();
        for (id, x, z, dir, by) in flying {
            for e in self.enemies.iter_mut() {
                if e.id != id && e.body.vulnerable() && e.body.act != Act::Held && (e.body.x - x).abs() < 44.0 && (e.body.z - z).abs() < 18.0 {
                    let armor = e.armor > 0;
                    e.body.take_hit(10, true, dir, armor);
                    e.last_by = by;
                    stunned(e);
                    self.events.push(Event::Hit { x: e.body.x, y: e.body.z - 60.0, big: true, by });
                }
            }
        }
        for (x, z, y, vx, vz, kind, dmg, knock) in new_shots {
            let id = self.new_id();
            self.shots.push(Shot { id, kind, x, z, y, vx, vz, life: 200, dmg, knock, owner: None, hits: Vec::new() });
            self.events.push(Event::Shoot { x, y: z - y });
        }
        for k in 0..summons {
            let spawn = Spawn { kind: if k % 2 == 0 { Kind::Thug } else { Kind::Knife }, elite: false, from_right: k % 2 == 0, delay: 0, z: self.rng.gen_range(Z_TOP..Z_BOTTOM) };
            self.pending.push((10 + k * 20, spawn));
        }
        // 倒下的敵人閃一閃就消失
        self.enemies.retain(|e| !(e.body.act == Act::Dead && e.body.t > 60));
        if let Some(id) = self.boss {
            if !self.enemies.iter().any(|e| e.id == id) {
                self.boss = None;
            }
        }
    }

    fn spawn_pending(&mut self) {
        for p in self.pending.iter_mut() {
            p.0 -= 1;
        }
        while let Some(pos) = self.pending.iter().position(|p| p.0 <= 0) {
            if self.enemies.iter().filter(|e| e.body.alive()).count() >= ENEMY_CAP {
                break;
            }
            let (_, s) = self.pending.remove(pos);
            let e = self.make_enemy(&s);
            if e.kind == Kind::Boss {
                self.boss = Some(e.id);
                self.events.push(Event::Boss);
            }
            self.enemies.push(e);
        }
    }

    fn make_enemy(&mut self, s: &Spawn) -> Enemy {
        let (hp_m, pow_m) = match self.difficulty {
            Difficulty::Easy => (0.8, 0.7),
            Difficulty::Normal => (1.0, 1.0),
            Difficulty::Hard => (1.2, 1.35),
        };
        let world_m = 1.0 + self.world as f32 * 0.12;
        let crowd = 1.0 + 0.35 * (self.players.len().max(1) - 1) as f32;
        let (hp, speed, power) = match s.kind {
            Kind::Thug => (38.0, 1.7, 1.0),
            Kind::Knife => (30.0, 2.5, 1.0),
            Kind::Brute => (85.0, 1.15, 1.0),
            Kind::Thrower => (34.0, 1.9, 1.0),
            Kind::Boss => ((380.0 + self.world as f32 * 70.0) * crowd, 1.9, 1.1),
        };
        let elite = if s.elite { 3.2 } else { 1.0 };
        let hp = (hp * hp_m * world_m * elite) as i32;
        let x = if s.from_right { self.camera + W + 50.0 } else { self.camera - 50.0 };
        let id = self.new_id();
        let mut body = Body::new(x, s.z, hp, if s.from_right { -1.0 } else { 1.0 });
        body.set(Act::Walk, 0);
        Enemy {
            id,
            kind: s.kind,
            elite: s.elite,
            body,
            brain: Brain { cool: 40 + self.rng.gen_range(0..40), offset: if id % 2 == 0 { -50.0 } else { 50.0 }, ..Brain::default() },
            speed: speed * (if s.elite { 1.1 } else { 1.0 }) * (1.0 + self.world as f32 * 0.03),
            power: power * pow_m * world_m.sqrt() * (if s.elite { 1.3 } else { 1.0 }),
            last_by: None,
            armor: 0,
            thrown_dmg: 0,
        }
    }

    // ---------- 打擊判定 ----------

    fn resolve_player_attacks(&mut self) {
        for p in self.players.iter_mut() {
            if p.out {
                continue;
            }
            let sp = p.special;
            let special = p.body.mv == Move::Special && p.body.act == Act::Attack;
            let mut hit_stop = 0;
            let d = if special { special_window(&p.body, &sp) } else { p.body.active_move() };
            let Some(d) = d else { continue };
            let slot = p.slot;
            let mut landed = false;
            // 多段的必殺：每隔幾格可以再打中同一個人
            if special && sp.rehit > 0 && (p.body.t - sp.startup) % sp.rehit == 0 && p.body.t > sp.startup {
                p.body.hit_list.clear();
            }
            // 多段的必殺只有最後一段把人打倒，前面幾段讓敵人硬直
            let last_hit = !special || sp.rehit == 0 || p.body.t + sp.rehit >= sp.startup + sp.active;
            for e in self.enemies.iter_mut() {
                let eb = &e.body;
                if !eb.vulnerable() || eb.act == Act::Held || !e.brain.entered || p.body.hit_list.contains(&e.id) {
                    continue;
                }
                if !p.body.reaches(&d, eb.x, eb.z, eb.y, e.body_r()) {
                    continue;
                }
                p.body.hit_list.push(e.id);
                landed = true;
                if !special {
                    let big = matches!(p.body.mv, Move::Kick | Move::DashAttack | Move::JumpKick | Move::Bat);
                    gain_sp(p, if big { SP_BIG } else { SP_HIT });
                }
                let dmg = (d.dmg * p.stats.power) as i32;
                let dir = if d.omni { (e.body.x - p.body.x).signum() } else { p.body.facing };
                let was_alive = e.body.hp > 0;
                // 大隻佬被拳頭打不會硬直（要用踢、跳踢、衝刺、丟人去撞）
                let fists = matches!(p.body.mv, Move::Jab | Move::Jab2) && e.kind == Kind::Brute;
                let armor = e.armor > 0 || fists;
                let knock = d.knock && last_hit;
                e.body.take_hit(dmg, knock, dir, armor);
                e.last_by = Some(slot);
                stunned(e);
                if !special {
                    hit_stop = hit_stop.max(if knock { HITSTOP_BIG } else { HITSTOP });
                }
                if special && knock && !armor && e.body.act == Act::Fall {
                    if sp.push > 0.0 {
                        e.body.vx = dir * sp.push;
                    }
                    if sp.lift > 0.0 {
                        e.body.vy = sp.lift;
                    }
                    // 爆裂重拳：被打飛的人會撞倒別人
                    if sp.launch && e.body.hp > 0 && e.grabbable() {
                        e.body.set(Act::Thrown, 0);
                        e.body.vx = dir * 9.0;
                        e.body.vy = 6.5;
                        e.body.y = e.body.y.max(8.0);
                        e.thrown_dmg = 0;
                    }
                }
                if e.kind == Kind::Boss && d.knock && !armor {
                    // 魔王被打倒後，起身會有一段霸體（不然會被連到死）
                    e.body.vx *= 0.5;
                }
                p.score += dmg as u32 * 10;
                self.events.push(Event::Hit { x: e.body.x, y: e.body.z - e.body.y - 70.0, big: d.knock, by: Some(slot) });
                if was_alive && e.body.hp <= 0 {
                    p.score += e.score();
                    p.kos += 1;
                    self.events.push(Event::Ko { x: e.body.x, z: e.body.z, boss: e.kind == Kind::Boss, by: Some(slot) });
                }
            }
            // 箱子
            for pr in self.props.iter_mut() {
                if pr.hp <= 0 || p.body.hit_list.contains(&pr.id) || !p.body.reaches(&d, pr.x, pr.z, 0.0, PROP_R) {
                    continue;
                }
                p.body.hit_list.push(pr.id);
                pr.hp -= 1;
                pr.flash = 6;
                self.events.push(Event::Hit { x: pr.x, y: pr.z - 40.0, big: false, by: Some(slot) });
            }
            if landed {
                // 武器每一揮第一次打中扣一次（一揮打中好幾個人也只算一次），用完就壞掉
                let first = !p.body.connected;
                p.body.connected = true;
                if first && matches!(p.body.mv, Move::Bat | Move::Blade) {
                    p.weapon_uses -= 1;
                    if p.weapon_uses <= 0 && p.weapon.take().is_some() {
                        self.events.push(Event::WeaponBreak { x: p.body.x + p.body.facing * 50.0, z: p.body.z });
                    }
                }
            }
            self.hitstop = self.hitstop.max(hit_stop);
        }
        // 打破的箱子掉出道具
        let mut drops = Vec::new();
        self.props.retain(|pr| {
            if pr.hp <= 0 {
                drops.push((pr.x, pr.z, pr.drop));
                false
            } else {
                true
            }
        });
        for pr in self.props.iter_mut() {
            pr.flash = (pr.flash - 1).max(0);
        }
        for (x, z, kind) in drops {
            let id = self.new_id();
            self.items.push(Item { id, kind, x, z, y: 30.0, vy: 4.0, uses: kind.uses() });
            self.events.push(Event::Break { x, z });
        }
    }

    fn resolve_enemy_attacks(&mut self) {
        for e in self.enemies.iter_mut() {
            let charge = e.body.act == Act::Charge;
            let d = if charge {
                Some(MoveData { dmg: 13.0, ..md(0, 1, 0, 13.0, 50.0, 22.0, true) })
            } else {
                e.body.active_move()
            };
            let Some(d) = d else { continue };
            for p in self.players.iter_mut() {
                let pb = &p.body;
                if p.out || !pb.vulnerable() || e.body.hit_list.contains(&(p.slot as u32)) {
                    continue;
                }
                if !e.body.reaches(&d, pb.x, pb.z, pb.y, PLAYER_R) {
                    continue;
                }
                e.body.hit_list.push(p.slot as u32);
                let dmg = (d.dmg * e.power) as i32;
                // 抓著敵人時被打：放開（被抓的敵人在下面發現沒人抓了會站起來）
                p.body.hold = None;
                p.body.take_hit(dmg, d.knock, e.body.facing, false);
                self.events.push(Event::Hurt { slot: p.slot, x: p.body.x, y: p.body.z - p.body.y - 70.0, big: d.knock });
            }
        }
        // 玩家放開了：被抓的敵人也要放開
        let holding: Vec<u32> = self.players.iter().filter(|p| p.body.act == Act::Grab).filter_map(|p| p.body.hold).collect();
        for e in self.enemies.iter_mut() {
            if e.body.act == Act::Held && !holding.contains(&e.id) {
                e.body.hold = None;
                e.body.set(Act::Stand, 0);
            }
        }
    }

    fn update_shots(&mut self) {
        let (cam, events) = (self.camera, &mut self.events);
        for s in self.shots.iter_mut() {
            s.x += s.vx;
            s.z = (s.z + s.vz).clamp(Z_TOP, Z_BOTTOM);
            s.life -= 1;
            if let Some(slot) = s.owner {
                for e in self.enemies.iter_mut() {
                    let eb = &e.body;
                    if !eb.vulnerable() || eb.act == Act::Held || !e.brain.entered || s.hits.contains(&e.id) || (eb.x - s.x).abs() > 40.0 || (eb.z - s.z).abs() > 22.0 {
                        continue;
                    }
                    s.hits.push(e.id);
                    let was_alive = e.body.hp > 0;
                    let armor = e.armor > 0;
                    e.body.take_hit(s.dmg, s.knock, s.vx.signum(), armor);
                    e.last_by = Some(slot);
                    stunned(e);
                    events.push(Event::Hit { x: e.body.x, y: e.body.z - 70.0, big: true, by: Some(slot) });
                    if let Some(p) = self.players.iter_mut().find(|p| p.slot == slot) {
                        p.score += s.dmg as u32 * 10;
                        if was_alive && e.body.hp <= 0 {
                            p.score += e.score();
                            p.kos += 1;
                            events.push(Event::Ko { x: e.body.x, z: e.body.z, boss: e.kind == Kind::Boss, by: Some(slot) });
                        }
                    }
                }
                if s.x < cam - 80.0 || s.x > cam + W + 80.0 {
                    s.life = 0;
                }
                continue;
            }
            for p in self.players.iter_mut() {
                let pb = &p.body;
                if p.out || !pb.vulnerable() || (pb.x - s.x).abs() > 28.0 || (pb.z - s.z).abs() > 16.0 || pb.y > s.y + 20.0 {
                    continue;
                }
                p.body.take_hit(s.dmg, s.knock, s.vx.signum(), false);
                events.push(Event::Hurt { slot: p.slot, x: p.body.x, y: p.body.z - 70.0, big: s.knock });
                s.life = 0;
                break;
            }
            if s.x < cam - 80.0 || s.x > cam + W + 80.0 {
                s.life = 0;
            }
        }
        self.shots.retain(|s| s.life > 0);
    }

    fn update_items(&mut self) {
        for it in self.items.iter_mut() {
            if it.y > 0.0 || it.vy > 0.0 {
                it.y += it.vy;
                it.vy -= G;
                if it.y <= 0.0 {
                    it.y = 0.0;
                    it.vy = 0.0;
                }
            }
        }
        let mut taken = Vec::new();
        for p in self.players.iter_mut() {
            if p.out || !matches!(p.body.act, Act::Stand | Act::Walk) {
                continue;
            }
            for it in self.items.iter() {
                if it.kind.is_weapon() || it.y > 0.0 || taken.contains(&it.id) || (it.x - p.body.x).abs() > 32.0 || (it.z - p.body.z).abs() > 16.0 {
                    continue;
                }
                // 血滿就不吃補血的（留給隊友）
                let heal = match it.kind {
                    ItemKind::Apple => p.body.max_hp / 4,
                    ItemKind::Meat => p.body.max_hp * 3 / 5,
                    ItemKind::Feast => p.body.max_hp,
                    ItemKind::Gem | ItemKind::Bat | ItemKind::Blade => 0,
                };
                if heal > 0 && p.body.hp >= p.body.max_hp {
                    continue;
                }
                p.body.hp = (p.body.hp + heal).min(p.body.max_hp);
                if it.kind == ItemKind::Gem {
                    p.score += 2000;
                }
                taken.push(it.id);
                self.events.push(Event::Pickup { slot: p.slot, kind: it.kind });
            }
        }
        self.items.retain(|it| !taken.contains(&it.id));
    }

    // ---------- 鏡頭與關卡進度 ----------

    fn update_camera(&mut self) {
        let alive: Vec<f32> = self.active_players().map(|p| p.body.x).collect();
        if alive.is_empty() || self.locked {
            return;
        }
        let front = alive.iter().cloned().fold(f32::MIN, f32::max);
        let back = alive.iter().cloned().fold(f32::MAX, f32::min);
        // 最前面的人推著鏡頭走，但不能把最後面的人丟出畫面
        let want = (front - W * 0.55).min(back - 40.0);
        let next_lock = self.def.waves.get(self.wave).map(|w| w.at).unwrap_or(self.length - W);
        let limit = next_lock.min(self.length - W);
        if want > self.camera {
            self.camera = (self.camera + (want - self.camera).min(6.0)).min(limit);
        }
        if self.go && self.camera - self.go_from > 260.0 {
            self.go = false;
        }
    }

    fn update_waves(&mut self) {
        let alive_enemies = self.enemies.iter().filter(|e| e.body.alive()).count();
        if self.wave_on {
            if self.pending.is_empty() && alive_enemies == 0 {
                self.wave_on = false;
                self.locked = false;
                self.wave += 1;
                if self.wave >= self.def.waves.len() {
                    // 最後一波打完：過關
                    self.phase = Phase::Clear;
                    self.timer = CLEAR_FRAMES;
                    for p in &mut self.players {
                        if !p.out && p.body.alive() {
                            p.body.hold = None;
                            p.body.set(Act::Win, 0);
                            p.body.y = 0.0;
                        }
                    }
                    self.shots.clear();
                    self.events.push(Event::Clear { world: self.world, sub: self.sub });
                } else {
                    self.go = true;
                    self.go_from = self.camera;
                    self.events.push(Event::Go);
                }
            }
            return;
        }
        if let Some(w) = self.def.waves.get(self.wave) {
            if self.camera >= w.at - 0.5 {
                self.camera = w.at;
                self.locked = true;
                self.wave_on = true;
                self.go = false;
                self.pending = w.spawns.iter().map(|s| (s.delay, s.clone())).collect();
                self.events.push(Event::Wave);
            }
        }
    }

    fn check_over(&mut self) {
        if self.players.iter().all(|p| p.out) {
            self.phase = Phase::Over;
            self.timer = 0;
            self.events.push(Event::Over);
        }
    }
}

/// 出拳時：前方打得到的距離內、上下差一點點的敵人，自動往他那邊對齊（上下很難完全對準）
const ALIGN_RANGE: f32 = 48.0;
const ALIGN_STEP: f32 = 18.0;
/// 玩家、箱子的身體半寬（敵人打玩家只算一點點，不要比以前容易被打到）
const PLAYER_R: f32 = 8.0;
const PROP_R: f32 = 22.0;
fn align_z(b: &mut Body, foes: &[(f32, f32)]) {
    let near = foes
        .iter()
        .filter(|(x, z)| {
            let ahead = (x - b.x) * b.facing;
            ahead > -10.0 && ahead < 120.0 && (z - b.z).abs() < ALIGN_RANGE
        })
        .min_by(|a, c| (a.0 - b.x).abs().total_cmp(&(c.0 - b.x).abs()));
    if let Some(&(_, z)) = near {
        b.z += (z - b.z).clamp(-ALIGN_STEP, ALIGN_STEP);
    }
}

/// 打中的停頓格數（一般／把人打倒）
const HITSTOP: i32 = 3;
const HITSTOP_BIG: i32 = 6;
/// 敵人被打到的硬直格數（玩家連段三下中間不會被他掙脫）
const ENEMY_HURT: i32 = 22;
/// 敵人被打之後至少等多久才會出手
const ENEMY_RECOVER: i32 = 40;

/// 敵人被打中：硬直長一點，恢復後也不會立刻反擊
fn stunned(e: &mut Enemy) {
    if e.body.act == Act::Hurt {
        e.body.dur = ENEMY_HURT;
    }
    e.brain.cool = e.brain.cool.max(ENEMY_RECOVER);
}

/// 必殺氣：滿了才能放必殺，放一次用光
pub const SP_MAX: i32 = 100;
/// 開場、掉命回來時有半條，馬上就能靠攻擊集滿
const SP_START: i32 = 50;
/// 普通攻擊打中一個敵人
const SP_HIT: i32 = 5;
/// 連段第三下、衝刺攻擊、跳踢、膝擊、丟人打中
const SP_BIG: i32 = 9;
/// 自己被打
const SP_HURT: i32 = 6;

fn gain_sp(p: &mut Player, n: i32) {
    p.sp = (p.sp + n).min(SP_MAX);
}

/// 開始放必殺
fn begin_special(b: &mut Body, sp: &Special) {
    b.attack(Move::Special);
    b.dur = sp.startup + sp.active + sp.recover;
    b.inv = sp.inv;
    if sp.leap > 0.0 {
        b.vy = sp.leap;
        b.y = 1.0;
    }
}

/// 必殺這一格有沒有打擊範圍（依角色）
fn special_window(b: &Body, sp: &Special) -> Option<MoveData> {
    if sp.wave || b.t < sp.startup || b.t >= sp.startup + sp.active {
        return None;
    }
    let (far, omni) = match sp.reach {
        Reach::Around(r) => (r, true),
        Reach::Front(f) => (f, false),
    };
    Some(MoveData { startup: sp.startup, active: sp.active, recover: sp.recover, dmg: sp.dmg, near: -10.0, far, zr: sp.zr, knock: true, omni })
}

/// 魔王的招式組合（每一大關不同）
struct BossKit {
    charge: bool,
    leap: bool,
    shots: u8,
    shot: ShotKind,
    teleport: bool,
}

fn boss_kit(world: usize) -> BossKit {
    match world {
        // 白色都市：白銀幫老大，衝撞
        0 => BossKit { charge: true, leap: false, shots: 0, shot: ShotKind::Rock, teleport: false },
        // 草原森林：森林巨熊，跳起來砸地
        1 => BossKit { charge: false, leap: true, shots: 0, shot: ShotKind::Rock, teleport: false },
        // 水下世界：鯊魚將軍，衝撞＋泡泡
        2 => BossKit { charge: true, leap: false, shots: 3, shot: ShotKind::Bubble, teleport: false },
        // 冰山：冰雪女王，冰柱＋跳躍
        3 => BossKit { charge: false, leap: true, shots: 2, shot: ShotKind::Ice, teleport: false },
        // 火炎礦坑：熔岩巨人，砸地＋火球
        4 => BossKit { charge: true, leap: true, shots: 3, shot: ShotKind::Fire, teleport: false },
        // 邪惡城堡：魔王，瞬移＋暗黑彈＋召喚
        _ => BossKit { charge: false, leap: false, shots: 5, shot: ShotKind::Dark, teleport: true },
    }
}
