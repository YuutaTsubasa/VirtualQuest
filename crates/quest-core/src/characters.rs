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

/// 必殺技的打擊範圍
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reach {
    /// 身邊一圈（半徑）
    Around(f32),
    /// 面前（最遠到哪）
    Front(f32),
}

/// 每個角色自己的必殺技：都用同一格「必殺」動作圖，差別在範圍、移動、段數、擊飛與特殊效果
#[derive(Clone, Copy, Debug)]
pub struct Special {
    pub startup: i32,
    pub active: i32,
    pub recover: i32,
    pub dmg: f32,
    pub reach: Reach,
    /// 前後（z）差多少以內打得到
    pub zr: f32,
    /// 多段：每隔幾格可以再打中同一個人一次（0 = 每人一次）
    pub rehit: i32,
    /// 擊飛的水平速度、往上的速度（0 = 一般倒地）
    pub push: f32,
    pub lift: f32,
    /// 出招時自己往前衝的速度、往上跳的速度
    pub dash: f32,
    pub leap: f32,
    /// 往前放一道會貫穿的氣波（自己身上沒有打擊範圍）
    pub wave: bool,
    /// 打中的敵人被打飛出去，像被丟出去一樣撞倒其他敵人
    pub launch: bool,
    /// 出招後幾格無敵
    pub inv: i32,
    /// 震地（畫面搖晃）
    pub quake: bool,
}

const BASE: Special = Special {
    startup: 3,
    active: 14,
    recover: 14,
    dmg: 8.0,
    reach: Reach::Around(100.0),
    zr: 24.0,
    rehit: 7,
    push: 0.0,
    lift: 0.0,
    dash: 0.0,
    leap: 0.0,
    wave: false,
    launch: false,
    inv: 20,
    quake: false,
};

/// 角色 id → 必殺技（名稱、說明在前端 src/lib/game/characters.js）
pub fn special_of(id: &str) -> Special {
    match id {
        // 旋風腿：原地轉兩圈，打身邊一圈（標準型）
        "xiaocui" => BASE,
        // 月光波：往前放一道貫穿的氣波，遠距離
        "yuehua" => Special { startup: 10, active: 1, recover: 16, dmg: 14.0, reach: Reach::Front(0.0), rehit: 0, wave: true, inv: 10, ..BASE },
        // 燃燒拳：往前衝的一記上勾拳，把人打上天
        "aming" => Special { startup: 6, active: 10, recover: 18, dmg: 22.0, reach: Reach::Front(90.0), zr: 20.0, rehit: 0, lift: 10.0, push: 3.0, dash: 6.0, inv: 16, ..BASE },
        // 糖果旋風：一邊往前一邊連轉，很多下小傷害
        "tangtang" => Special { startup: 3, active: 30, recover: 10, dmg: 4.0, reach: Reach::Around(80.0), zr: 20.0, rehit: 6, dash: 2.5, inv: 24, ..BASE },
        // 殺球：跳起來往前下方扣殺
        "xiaochun" => Special { startup: 4, active: 26, recover: 10, dmg: 16.0, reach: Reach::Front(85.0), zr: 26.0, rehit: 0, dash: 4.0, leap: 8.0, inv: 14, ..BASE },
        // 推手：把面前一整排敵人推得老遠
        "lanjie" => Special { startup: 8, active: 6, recover: 14, dmg: 8.0, reach: Reach::Front(140.0), zr: 40.0, rehit: 0, push: 10.0, inv: 12, ..BASE },
        // 狼牙突進：往前衝過去，路上的敵人連咬好幾下
        "cangya" => Special { startup: 4, active: 16, recover: 12, dmg: 6.0, reach: Reach::Front(60.0), zr: 22.0, rehit: 5, dash: 9.0, inv: 20, ..BASE },
        // 豪賭震地：起手很慢，但震倒身邊很大一圈
        "jinye" => Special { startup: 18, active: 4, recover: 20, dmg: 16.0, reach: Reach::Around(160.0), zr: 44.0, rehit: 0, inv: 22, quake: true, ..BASE },
        // 騎士衝鋒：無敵地往前衝撞，把人撞開
        "leo" => Special { startup: 3, active: 18, recover: 12, dmg: 12.0, reach: Reach::Front(70.0), zr: 24.0, rehit: 0, push: 6.0, dash: 6.0, inv: 24, ..BASE },
        // 爆裂重拳：一拳把人打飛出去，撞倒後面的敵人
        "alie" => Special { startup: 9, active: 5, recover: 18, dmg: 20.0, reach: Reach::Front(85.0), zr: 20.0, rehit: 0, launch: true, inv: 12, ..BASE },
        // 天翼十字斬：小跳一下，前後一起斬兩次
        "ayi" => Special { startup: 4, active: 16, recover: 12, dmg: 12.0, reach: Reach::Around(115.0), zr: 30.0, rehit: 8, leap: 5.0, inv: 22, ..BASE },
        _ => BASE,
    }
}
