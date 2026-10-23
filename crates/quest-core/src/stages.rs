//! 關卡表：六大關 × 三小關。每一小關是一段往右捲的路，路上有幾個「關卡鎖」：
//! 鏡頭走到那裡就停住、敵人從左右兩邊進場，全部打倒才會出現「GO →」繼續走。
//! 小關 1、2 的最後一波有一隻強化的頭目（elite），小關 3 是大魔王。
//!
//! 關名、敵人的名字與配色在前端（src/lib/game/stages.js），這裡只管數量與種類。

use crate::ai::Difficulty;
use crate::game::{ItemKind, Kind, W, Z_BOTTOM, Z_TOP};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

pub const WORLDS: usize = 6;
pub const SUBS: usize = 3;

#[derive(Clone, Debug)]
pub struct Spawn {
    pub kind: Kind,
    pub elite: bool,
    /// 從畫面右邊進場（否則左邊）
    pub from_right: bool,
    /// 這一波開始後第幾格出現
    pub delay: i32,
    pub z: f32,
}

#[derive(Clone, Debug)]
pub struct Wave {
    /// 鏡頭左邊走到這裡就鎖住
    pub at: f32,
    pub spawns: Vec<Spawn>,
}

#[derive(Clone, Debug)]
pub struct StageDef {
    pub length: f32,
    pub waves: Vec<Wave>,
    /// 可以打破的箱子：(x, z, 掉出來的東西)
    pub props: Vec<(f32, f32, ItemKind)>,
}

/// 這一大關出現哪些雜兵（越後面越多種）
fn pool(world: usize, sub: usize) -> Vec<Kind> {
    let mut p = vec![Kind::Thug, Kind::Thug];
    if world > 0 || sub > 0 {
        p.push(Kind::Knife);
    }
    // 白色都市：1-2 開始有投擲手、1-3 有大隻佬（每一大關的四種敵人都會出場）
    if world >= 1 || sub >= 1 {
        p.push(Kind::Thrower);
    }
    if world >= 1 || sub >= 2 {
        p.push(Kind::Brute);
    }
    if world >= 4 {
        p.push(Kind::Knife);
        p.push(Kind::Thrower);
    }
    p
}

/// players：參加的人數（敵人跟著變多）
pub fn build_stage(world: usize, sub: usize, players: usize, difficulty: Difficulty) -> StageDef {
    let mut rng = SmallRng::seed_from_u64((world * 10 + sub) as u64 * 7919 + 17);
    let length = W * [3.4, 3.8, 2.6][sub];
    let boss_stage = sub == 2;
    let n_waves = if boss_stage { 3 } else if world >= 3 { 4 } else { 3 };
    let kinds = pool(world, sub);
    let extra = match difficulty {
        Difficulty::Easy => -1,
        Difficulty::Normal => 0,
        Difficulty::Hard => 1,
    };
    let crowd = 1.0 + 0.3 * (players.max(1) - 1) as f32;

    let mut waves = Vec::new();
    for i in 0..n_waves {
        let last = i + 1 == n_waves;
        let at = if last { length - W } else { (length - W) * (i + 1) as f32 / n_waves as f32 - 140.0 };
        let base = 3 + world as i32 / 2 + i as i32 / 2 + extra;
        let count = ((base.max(2) as f32) * crowd).round() as usize;
        let mut spawns = Vec::new();
        if last && boss_stage {
            // 大魔王＋兩隻跟班
            spawns.push(Spawn { kind: Kind::Boss, elite: false, from_right: true, delay: 30, z: (Z_TOP + Z_BOTTOM) / 2.0 });
            for k in 0..(1 + players / 2) {
                spawns.push(Spawn { kind: Kind::Thug, elite: false, from_right: k % 2 == 1, delay: 90 + k as i32 * 50, z: rng.gen_range(Z_TOP..Z_BOTTOM) });
            }
        } else {
            for k in 0..count {
                let kind = kinds[rng.gen_range(0..kinds.len())];
                spawns.push(Spawn {
                    kind,
                    elite: false,
                    from_right: k % 3 != 1,
                    delay: (k as i32) * 45 + rng.gen_range(0..20),
                    z: rng.gen_range(Z_TOP + 10.0..Z_BOTTOM - 10.0),
                });
            }
            if last {
                // 小頭目：大隻佬或刀手的強化版
                let kind = if sub == 1 { Kind::Brute } else { Kind::Knife };
                spawns.push(Spawn { kind, elite: true, from_right: true, delay: count as i32 * 45 + 30, z: (Z_TOP + Z_BOTTOM) / 2.0 });
            }
        }
        waves.push(Wave { at: at.max(0.0), spawns });
    }

    // 路上的箱子：補血食物或寶石
    let mut props = Vec::new();
    let n_props = 2 + rng.gen_range(0..2) + sub.min(1);
    for k in 0..n_props {
        let x = 420.0 + (length - 700.0) * (k as f32 + 0.5) / n_props as f32 + rng.gen_range(-80.0..80.0);
        let z = rng.gen_range(Z_TOP + 15.0..Z_BOTTOM - 15.0);
        let item = match rng.gen_range(0..10) {
            0..=3 => ItemKind::Apple,
            4..=6 => ItemKind::Meat,
            7 => ItemKind::Feast,
            _ => ItemKind::Gem,
        };
        props.push((x, z, item));
    }
    StageDef { length, waves, props }
}
