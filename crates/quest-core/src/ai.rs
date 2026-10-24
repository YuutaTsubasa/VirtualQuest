//! 電腦隊友：找最近的敵人、走到同一條線上、靠近就連打；被圍住時放必殺技，血少時去撿補血。
//! 沒有敵人就跟著往右走（打箱子、跟上「GO →」）。
//! 輸出跟真人一樣的 `Input`，所以模擬不用分辨誰是電腦。

use crate::game::{Act, ItemKind, Match, Phase, Input, W};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

struct Cfg {
    /// 每隔幾格才重新判斷一次（越大越遲鈍）
    reaction: i32,
    /// 兩下攻擊之間至少隔幾格
    attack_gap: i32,
    /// 被圍住時放必殺技的機率
    special: f64,
}

impl Difficulty {
    fn cfg(self) -> Cfg {
        match self {
            Difficulty::Easy => Cfg { reaction: 14, attack_gap: 20, special: 0.2 },
            Difficulty::Normal => Cfg { reaction: 7, attack_gap: 11, special: 0.5 },
            Difficulty::Hard => Cfg { reaction: 3, attack_gap: 7, special: 0.85 },
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "簡單",
            Difficulty::Normal => "普通",
            Difficulty::Hard => "困難",
        }
    }
}

pub struct Ai {
    /// 控制第幾個玩家（Match.players 的索引）
    index: usize,
    difficulty: Difficulty,
    rng: SmallRng,
    think: i32,
    last_attack: i32,
    special_cool: i32,
    /// 上次決定要去的位置
    goal: Option<(f32, f32)>,
    /// 想打的方向（-1／1），0 = 不打
    fight: f32,
    jump_kick: i32,
    /// 跑步用的連按兩下：還剩幾格（放開、按、放開，之後照常按住就跑起來）
    dash_tap: i32,
}

impl Ai {
    pub fn new(index: usize, difficulty: Difficulty, seed: u64) -> Self {
        Ai {
            index,
            difficulty,
            rng: SmallRng::seed_from_u64(seed),
            think: 0,
            last_attack: 0,
            special_cool: 0,
            goal: None,
            fight: 0.0,
            jump_kick: 0,
            dash_tap: 0,
        }
    }

    pub fn think(&mut self, m: &Match) -> Input {
        let mut out = Input::default();
        let Some(me) = m.players.get(self.index) else { return out };
        if me.out || !matches!(m.phase, Phase::Play | Phase::Intro) {
            return out;
        }
        let b = &me.body;
        let cfg = self.difficulty.cfg();
        self.last_attack += 1;
        self.special_cool = (self.special_cool - 1).max(0);

        // 跳踢：起跳後幾格再按攻擊
        if self.jump_kick > 0 {
            self.jump_kick -= 1;
            if self.jump_kick == 0 && b.act == Act::Jump {
                out.attack_pressed = true;
            }
            return out;
        }
        // 抓住了：膝擊三下（第三下自動丟出去）
        if b.act == Act::Grab {
            if self.last_attack >= cfg.attack_gap {
                self.last_attack = 0;
                out.attack_pressed = true;
            }
            return out;
        }

        self.think -= 1;
        if self.think <= 0 {
            self.think = cfg.reaction + self.rng.gen_range(0..4);
            self.decide(m);
        }

        let Some((gx, gz)) = self.goal else { return out };
        let dx = gx - b.x;
        let dz = gz - b.z;
        if dx.abs() > 6.0 {
            out.left = dx < 0.0;
            out.right = dx > 0.0;
        }
        // 離目標很遠：連按兩下跑過去（跑步中按住同一個方向就會一直跑）
        if b.act == Act::Run && dx.abs() < 90.0 {
            out.left = false;
            out.right = false;
        } else if self.dash_tap > 0 {
            // 放開 → 按一下 → 放開，下一格照常按住就是第二下
            self.dash_tap -= 1;
            let press = self.dash_tap == 1;
            out.left = press && dx < 0.0;
            out.right = press && dx > 0.0;
        } else if dx.abs() > 260.0 && matches!(b.act, Act::Stand | Act::Walk) && self.rng.gen_bool(0.5) {
            self.dash_tap = 3;
            out.left = false;
            out.right = false;
        }
        if dz.abs() > 4.0 {
            out.up = dz < 0.0;
            out.down = dz > 0.0;
        }
        if self.fight != 0.0 && dx.abs() <= 8.0 && dz.abs() <= 7.0 {
            // 到位了：面向敵人連打
            out.left = false;
            out.right = false;
            out.up = false;
            out.down = false;
            // 攻擊時同時按方向＝原地轉身出拳（只按方向會走動，會在原地來回晃）
            if self.last_attack >= cfg.attack_gap && matches!(b.act, Act::Stand | Act::Walk) {
                self.last_attack = 0;
                out.attack_pressed = true;
                out.left = self.fight < 0.0;
                out.right = self.fight > 0.0;
            }
        }
        // 被圍住：必殺技
        let crowd = m
            .enemies
            .iter()
            .filter(|e| e.body.vulnerable() && (e.body.x - b.x).abs() < 90.0 && (e.body.z - b.z).abs() < 22.0)
            .count();
        if crowd >= 3 && self.special_cool == 0 && b.hp as f32 > b.max_hp as f32 * 0.25 && matches!(b.act, Act::Stand | Act::Walk) {
            self.special_cool = 200;
            if self.rng.gen_bool(cfg.special) {
                out = Input { special_pressed: true, ..Input::default() };
            }
        }
        out
    }

    fn decide(&mut self, m: &Match) {
        let me = &m.players[self.index].body;
        self.fight = 0.0;
        // 血少：去撿補血
        if me.hp < me.max_hp / 2 {
            if let Some(it) = m
                .items
                .iter()
                .filter(|it| it.kind != ItemKind::Gem && it.x > m.camera && it.x < m.camera + W)
                .min_by(|a, b| (a.x - me.x).abs().total_cmp(&(b.x - me.x).abs()))
            {
                self.goal = Some((it.x, it.z));
                return;
            }
        }
        // 最近的敵人（在畫面裡、打得到的）；已經有隊友在打的敵人算遠一點，大家才會分散開來
        let me_xz = (me.x, me.z);
        let mates: Vec<(f32, f32)> = m
            .players
            .iter()
            .enumerate()
            .filter(|(i, p)| *i != self.index && !p.out && p.body.alive())
            .map(|(_, p)| (p.body.x, p.body.z))
            .collect();
        let score = |e: &crate::game::Enemy| {
            let d = (e.body.x - me_xz.0).abs() + (e.body.z - me_xz.1).abs() * 2.0;
            let crowded = mates.iter().filter(|(x, z)| (x - e.body.x).abs() < 80.0 && (z - e.body.z).abs() < 20.0).count();
            d + crowded as f32 * 160.0
        };
        let target = m
            .enemies
            .iter()
            .filter(|e| e.body.alive() && e.body.act != Act::Held && e.body.x > m.camera + 10.0 && e.body.x < m.camera + W - 10.0)
            .min_by(|a, b| score(a).total_cmp(&score(b)));
        if let Some(e) = target {
            let mut side = if me.x < e.body.x { -1.0 } else { 1.0 };
            // 那一邊已經有隊友站著：換另一邊打
            if mates.iter().any(|(x, z)| (x - (e.body.x + side * 50.0)).abs() < 30.0 && (z - e.body.z).abs() < 16.0) {
                side = -side;
            }
            // 另一邊貼著畫面邊緣、站不進去：還是打這一邊
            let (lo, hi) = (m.camera + 30.0, m.camera + W - 30.0);
            let want = e.body.x + side * 48.0;
            if want < lo || want > hi {
                side = -side;
            }
            let dz_off = [-6.0, 6.0, -3.0, 3.0][self.index % 4];
            self.goal = Some(((e.body.x + side * 48.0).clamp(lo, hi), e.body.z + dz_off));
            self.fight = -side;
            // 偶爾從遠一點的地方跳踢
            let dx = (e.body.x - me.x).abs();
            if dx > 110.0 && dx < 170.0 && (e.body.z - me.z).abs() < 10.0 && self.rng.gen_bool(0.15) && matches!(me.act, Act::Stand | Act::Walk) {
                self.jump_kick = 9;
            }
            return;
        }
        // 箱子
        if let Some(pr) = m.props.iter().find(|p| p.x > m.camera + 20.0 && p.x < m.camera + W - 20.0) {
            let side = if me.x < pr.x { -1.0 } else { 1.0 };
            self.goal = Some((pr.x + side * 44.0, pr.z));
            self.fight = -side;
            return;
        }
        // 沒事做：跟上最前面的真人／往右走
        let lead = m
            .players
            .iter()
            .enumerate()
            .filter(|(i, p)| *i != self.index && !p.out)
            .map(|(_, p)| p.body.x)
            .fold(me.x, f32::max);
        let x = if m.locked { m.camera + W * 0.5 } else { (lead + 30.0).min(m.camera + W - 60.0) + if m.go { 120.0 } else { 0.0 } };
        let z = (crate::game::Z_TOP + crate::game::Z_BOTTOM) / 2.0 + (self.index as f32 - 1.5) * 36.0;
        self.goal = Some((x, z));
    }
}
