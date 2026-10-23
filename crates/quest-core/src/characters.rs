//! 角色能力值。id 要跟前端 src/lib/game/characters.js 一致（那邊放名字、台詞、圖片）。
//! 速度／力量／體力各 1–5，一般角色總和固定 10。

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stats {
    /// 走路速度（每格像素）
    pub speed: f32,
    /// 攻擊力倍率
    pub power: f32,
    /// 最大體力
    pub hp: i32,
}

/// (id, 速度, 力量, 體力, 隱藏角色)
pub const CHARACTERS: &[(&str, u8, u8, u8, bool)] = &[
    ("xiaocui", 4, 2, 4, false),
    ("yuehua", 3, 4, 3, false),
    ("aming", 2, 5, 3, false),
    ("tangtang", 5, 2, 3, false),
    ("xiaochun", 5, 3, 2, false),
    ("lanjie", 2, 4, 4, false),
    ("cangya", 4, 4, 2, false),
    ("jinye", 1, 5, 4, false),
    ("leo", 3, 3, 4, false),
    ("alie", 3, 5, 2, false),
    ("ayi", 4, 4, 3, true),
];

pub fn is_character(id: &str) -> bool {
    CHARACTERS.iter().any(|c| c.0 == id)
}

/// 能力值（1–5）換成實際數值；不認得的 id 用平均值
pub fn stats_of(id: &str) -> Stats {
    let (s, p, v) = CHARACTERS
        .iter()
        .find(|c| c.0 == id)
        .map(|c| (c.1, c.2, c.3))
        .unwrap_or((3, 3, 4));
    Stats {
        speed: 2.4 + s as f32 * 0.32,
        power: 0.75 + p as f32 * 0.1,
        hp: 90 + v as i32 * 10,
    }
}

/// 隨機挑一個非隱藏角色（電腦隊友用），盡量不跟別人重複
pub fn random_character(rng: &mut impl rand::Rng, avoid: &[&str]) -> &'static str {
    let mut pool: Vec<&str> = CHARACTERS
        .iter()
        .filter(|c| !c.4 && !avoid.contains(&c.0))
        .map(|c| c.0)
        .collect();
    if pool.is_empty() {
        pool = CHARACTERS.iter().filter(|c| !c.4).map(|c| c.0).collect();
    }
    pool[rng.gen_range(0..pool.len())]
}
