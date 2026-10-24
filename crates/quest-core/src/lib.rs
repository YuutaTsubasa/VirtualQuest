//! 虛擬闖天下的遊戲引擎。
//! - `game`：清版動作的模擬（確定性，同樣輸入同樣結果）
//! - `stages`：六大關 × 三小關的關卡表
//! - `ai`：電腦隊友
//! - `room`：連線房間（四個座位、電腦隊友、聊天）
//! - `protocol`：連線訊息的處理（區網伺服器與線上房間共用）
//! - `characters`：角色能力值

pub mod ai;
pub mod characters;
pub mod game;
pub mod protocol;
pub mod room;
pub mod stages;

#[cfg(test)]
mod tests {
    use crate::ai::{Ai, Difficulty};
    use crate::game::{Entrant, Event, Match, Phase};
    use crate::room::{Room, Settings};

    /// 全部交給電腦打，回傳打到哪一關、結束時的狀態
    fn run(chars: &[&str], world: usize, d: Difficulty, frames: usize, seed: u64) -> (Match, Vec<(usize, usize)>) {
        let entrants: Vec<Entrant> = chars.iter().enumerate().map(|(i, c)| Entrant { slot: i, character: c }).collect();
        let mut m = Match::new(&entrants, world, d, 3, seed);
        let mut ais: Vec<Ai> = (0..chars.len()).map(|i| Ai::new(i, Difficulty::Hard, seed + i as u64)).collect();
        let mut cleared = Vec::new();
        for _ in 0..frames {
            let inputs: Vec<_> = ais.iter_mut().map(|a| a.think(&m)).collect();
            m.step(&inputs);
            for e in &m.events {
                if let Event::Clear { world, sub } = e {
                    cleared.push((*world, *sub));
                }
            }
            if matches!(m.phase, Phase::Over | Phase::Ending) {
                break;
            }
        }
        (m, cleared)
    }

    #[test]
    fn cpu_team_clears_first_stages() {
        // 四個電腦隊友、簡單難度：至少要能打過 1-1、1-2
        let (m, cleared) = run(&["aming", "xiaocui", "jinye", "leo"], 0, Difficulty::Easy, 60 * 60 * 8, 1);
        assert!(cleared.len() >= 2, "應該至少過兩小關，實際 {:?}（停在 {}-{}，{:?}）", cleared, m.world + 1, m.sub + 1, m.phase);
    }

    #[test]
    fn boss_stage_has_boss() {
        let entrants = [Entrant { slot: 0, character: "aming" }];
        let mut m = Match::new(&entrants, 5, Difficulty::Normal, 3, 7);
        // 跳到 6-3：直接打完前兩小關太久，這裡只確認關卡表
        let def = crate::stages::build_stage(5, 2, 1, Difficulty::Normal);
        assert!(def.waves.last().unwrap().spawns.iter().any(|s| s.kind == crate::game::Kind::Boss));
        for _ in 0..200 {
            m.step(&[Default::default()]);
        }
        assert_eq!(m.phase, Phase::Play);
    }

    #[test]
    fn everyone_out_is_game_over_and_continue_works() {
        // 一個人站著不動：遲早被打倒、用完命
        let entrants = [Entrant { slot: 0, character: "tangtang" }];
        let mut m = Match::new(&entrants, 2, Difficulty::Hard, 1, 3);
        let mut walk = crate::game::Input::default();
        walk.right = true;
        for _ in 0..60 * 60 * 6 {
            m.step(&[walk]);
            if m.phase == Phase::Over {
                break;
            }
        }
        assert_eq!(m.phase, Phase::Over, "站著不打應該會輸");
        assert!(m.continue_game());
        assert_eq!(m.phase, Phase::Intro);
        assert_eq!(m.players[0].lives, 1);
        assert!(!m.players[0].out);
    }

    #[test]
    fn room_bots_and_disconnect() {
        let mut room = Room::new(Settings { difficulty: Difficulty::Easy, ..Settings::default() });
        let (seat, token) = room.join("悠太", Some("xiaocui")).unwrap();
        room.connect(seat);
        room.add_bot(None, Difficulty::Hard).unwrap();
        assert_eq!(room.seat_of_token(&token), Some(seat));
        room.start().unwrap();
        assert!(room.join("晚到的人", None).is_err());
        // 玩家斷線：電腦接手
        room.disconnect(seat);
        for _ in 0..600 {
            room.tick();
        }
        let frame = room.frame().unwrap();
        assert_eq!(frame["players"].as_array().unwrap().len(), 2);
        assert_eq!(frame["players"][0]["character"], "xiaocui");
        assert!(frame["players"][0]["hp"].is_number());
        room.reset_to_lobby();
        assert_eq!(room.view(Some(0))["phase"], "lobby");
    }

    #[test]
    fn protocol_host_and_guest() {
        use crate::protocol::{drop_conn, handle, room_message, Conn};
        use serde_json::json;
        let mut room = Room::default();
        let (mut host, mut guest) = (Conn::default(), Conn::default());
        let (r, changed) = handle(&mut room, "key", &mut host, &json!({"type": "hello", "name": "房主", "host_key": "key"}));
        assert!(changed && host.is_host && host.seat == Some(0));
        assert_eq!(r[0]["type"], "welcome");
        let (r, _) = handle(&mut room, "key", &mut guest, &json!({"type": "hello", "name": "客人", "host_key": "wrong"}));
        let token = r[0]["token"].as_str().unwrap().to_string();
        assert!(!guest.is_host && guest.seat == Some(1));
        let (r, _) = handle(&mut room, "key", &mut guest, &json!({"type": "host", "cmd": "start"}));
        assert_eq!(r[0]["type"], "error");
        handle(&mut room, "key", &mut host, &json!({"type": "host", "cmd": "add_bot", "seat": 3}));
        assert!(room.seats[3].is_some());
        handle(&mut room, "key", &mut host, &json!({"type": "host", "cmd": "start"}));
        assert!(room.playing());
        assert!(drop_conn(&mut room, &mut guest));
        let mut back = Conn::default();
        handle(&mut room, "key", &mut back, &json!({"type": "hello", "token": token}));
        assert_eq!(back.seat, Some(1));
        assert_eq!(room_message(&room, &back)["state"]["you"], 1);
        // 按一下攻擊（次數 +1）
        handle(&mut room, "key", &mut back, &json!({"type": "input", "b": 0, "s": 1}));
        assert!(room.tick());
    }
}

#[cfg(test)]
mod balance {
    use crate::ai::{Ai, Difficulty};
    use crate::game::{Entrant, Event, Match, Phase};

    /// cargo test --release balance -- --ignored --nocapture：看電腦隊伍能打到哪
    #[test]
    #[ignore]
    fn campaign_progress() {
        for (chars, d) in [(vec!["aming"], Difficulty::Normal), (vec!["aming", "xiaocui"], Difficulty::Normal), (vec!["aming", "xiaocui", "jinye", "leo"], Difficulty::Hard)] {
            for seed in 0..2 {
                let entrants: Vec<Entrant> = chars.iter().enumerate().map(|(i, c)| Entrant { slot: i, character: c }).collect();
                let mut m = Match::new(&entrants, 0, d, 3, seed);
                let mut ais: Vec<Ai> = (0..chars.len()).map(|i| Ai::new(i, Difficulty::Hard, seed * 9 + i as u64)).collect();
                let mut log = Vec::new();
                let mut f = 0;
                for _ in 0..60 * 60 * 60 {
                    f += 1;
                    let inputs: Vec<_> = ais.iter_mut().map(|a| a.think(&m)).collect();
                    m.step(&inputs);
                    for e in &m.events {
                        if let Event::Clear { world, sub } = e {
                            log.push(format!("{}-{}@{}s", world + 1, sub + 1, f / 60));
                        }
                    }
                    if matches!(m.phase, Phase::Over | Phase::Ending) {
                        break;
                    }
                }
                println!("{:?} {:?} seed {}: {:?} 結束在 {}-{} {:?} {}s", chars.len(), d, seed, log, m.world + 1, m.sub + 1, m.phase, f / 60);
            }
        }
    }

    /// 換血程度：打中幾下、被打幾下、在自己出招中被打斷幾次（互打）、連段打滿三下幾次
    #[test]
    #[ignore]
    fn trading() {
        let (mut hit, mut hurt, mut traded, mut finish, mut frames) = (0, 0, 0, 0, 0);
        // 出招幾次、幾次一下都沒打中
        let (mut swings, mut whiffs) = (0, 0);
        for (id, ..) in crate::characters::CHARACTERS.iter().take(10) {
            let entrants = [Entrant { slot: 0, character: id }];
            let mut m = Match::new(&entrants, 0, Difficulty::Normal, 3, 7);
            let mut ai = Ai::new(0, Difficulty::Hard, 11);
            for _ in 0..60 * 300 {
                let inp = ai.think(&m);
                let was = (m.players[0].body.act, m.players[0].body.mv);
                let was_t = m.players[0].body.t;
                let was_hit = m.players[0].body.connected;
                m.step(&[inp]);
                // 一招結束（換成別的動作或重新出招）時，看它有沒有打中
                let now = &m.players[0].body;
                if was.0 == crate::game::Act::Attack && matches!(was.1, crate::game::Move::Jab | crate::game::Move::Jab2 | crate::game::Move::Kick) && (now.act != crate::game::Act::Attack || now.t < was_t) {
                    swings += 1;
                    if !was_hit {
                        whiffs += 1;
                    }
                }
                frames += 1;
                for e in &m.events {
                    match e {
                        Event::Hit { by: Some(0), .. } => {
                            hit += 1;
                            if m.players[0].body.mv == crate::game::Move::Kick {
                                finish += 1;
                            }
                        }
                        Event::Hurt { slot: 0, .. } => {
                            hurt += 1;
                            if was.0 == crate::game::Act::Attack && was.1 != crate::game::Move::None {
                                traded += 1;
                            }
                        }
                        _ => {}
                    }
                }
                if m.world > 0 || m.phase == Phase::Over {
                    break;
                }
            }
        }
        println!("打中 {hit}、被打 {hurt}（每打中 10 下被打 {:.1} 下）、出招中被打斷 {traded}（佔被打 {:.0}%）、連段打滿 {finish}、出拳腳 {swings} 次揮空 {:.0}%、{} 秒", hurt as f32 * 10.0 / hit as f32, traded as f32 * 100.0 / hurt.max(1) as f32, whiffs as f32 * 100.0 / swings.max(1) as f32, frames / 60);
    }

    /// 每個角色自己打 1-1：必殺放了幾次、必殺打中幾下
    #[test]
    #[ignore]
    fn specials_per_character() {
        for (id, ..) in crate::characters::CHARACTERS {
            let entrants = [Entrant { slot: 0, character: id }];
            let mut m = Match::new(&entrants, 0, Difficulty::Normal, 3, 3);
            let mut ai = Ai::new(0, Difficulty::Hard, 5);
            let (mut used, mut hits, mut casting) = (0, 0, false);
            for _ in 0..60 * 600 {
                let inp = ai.think(&m);
                m.step(&[inp]);
                casting = m.players[0].body.mv == crate::game::Move::Special && m.players[0].body.act == crate::game::Act::Attack || (casting && !m.shots.is_empty());
                for e in &m.events {
                    match e {
                        Event::Special { .. } => used += 1,
                        Event::Hit { by: Some(0), .. } if casting => hits += 1,
                        _ => {}
                    }
                }
                if m.world > 0 || m.phase == Phase::Over {
                    break;
                }
            }
            println!("{id:9} 必殺 {used:2} 次、打中 {hits:3} 下、{}", format!("到 {}-{}", m.world + 1, m.sub + 1));
        }
    }
}


#[cfg(test)]
mod no_stall {
    use crate::ai::{Ai, Difficulty};
    use crate::game::{Entrant, Event, Match, Phase};

    /// 每一大關都從頭打：電腦隊伍不能卡住（要嘛過關、要嘛全軍覆沒，不能在同一波停太久）
    #[test]
    fn every_world_keeps_moving() {
        for world in 0..6 {
            let chars = ["aming", "xiaocui", "jinye", "leo"];
            let entrants: Vec<Entrant> = chars.iter().enumerate().map(|(i, c)| Entrant { slot: i, character: c }).collect();
            let mut m = Match::new(&entrants, world, Difficulty::Easy, 9, world as u64);
            let mut ais: Vec<Ai> = (0..4).map(|i| Ai::new(i, Difficulty::Hard, i as u64)).collect();
            let mut idle = 0;
            let mut cleared = 0;
            for _ in 0..60 * 60 * 12 {
                let inputs: Vec<_> = ais.iter_mut().map(|a| a.think(&m)).collect();
                m.step(&inputs);
                let progress = m.events.iter().any(|e| matches!(e, Event::Ko { .. } | Event::Hit { .. } | Event::Go | Event::Clear { .. } | Event::Wave));
                cleared += m.events.iter().filter(|e| matches!(e, Event::Clear { .. })).count();
                idle = if progress { 0 } else { idle + 1 };
                assert!(idle < 60 * 60, "第 {} 大關卡住了：停在 {}-{}，敵人 {:?}", world + 1, m.world + 1, m.sub + 1,
                    m.enemies.iter().map(|e| (e.kind, e.body.act, e.body.x as i32, e.body.z as i32, e.body.y as i32, e.body.hp, e.body.t)).collect::<Vec<_>>());
                if matches!(m.phase, Phase::Over | Phase::Ending) || m.world != world {
                    break;
                }
            }
            println!("第 {} 大關：過了 {} 小關，結束在 {}-{} {:?}", world + 1, cleared, m.world + 1, m.sub + 1, m.phase);
        }
    }
}

#[cfg(test)]
mod run {
    use crate::ai::Difficulty;
    use crate::game::{Act, Entrant, Input, Match, Move, Phase};

    fn press(right: bool, attack: bool, jump: bool) -> Input {
        Input { right, attack_pressed: attack, jump_pressed: jump, ..Input::default() }
    }

    /// 連按兩下右：跑起來、跑得比走路快；跑步中按攻擊是衝刺攻擊、按跳是跑步跳
    #[test]
    fn double_tap_runs() {
        let entrants = [Entrant { slot: 0, character: "xiaochun" }];
        let mut m = Match::new(&entrants, 0, Difficulty::Easy, 3, 1);
        while m.phase != Phase::Play {
            m.step(&[Input::default()]);
        }
        let x0 = m.players[0].body.x;
        for _ in 0..10 {
            m.step(&[press(true, false, false)]);
        }
        let walked = m.players[0].body.x - x0;
        assert_eq!(m.players[0].body.act, Act::Walk);
        // 放開、再按：跑
        m.step(&[Input::default()]);
        m.step(&[press(true, false, false)]);
        assert_eq!(m.players[0].body.act, Act::Run);
        let x1 = m.players[0].body.x;
        for _ in 0..10 {
            m.step(&[press(true, false, false)]);
        }
        assert!(m.players[0].body.x - x1 > walked * 1.5, "跑步應該比走路快");
        m.step(&[press(true, true, false)]);
        assert_eq!(m.players[0].body.mv, Move::DashAttack);
        // 等衝刺攻擊結束（打空收招比較久），再跑一次、跳
        for _ in 0..60 {
            m.step(&[Input::default()]);
        }
        m.step(&[press(true, false, false)]);
        m.step(&[Input::default()]);
        m.step(&[press(true, false, false)]);
        assert_eq!(m.players[0].body.act, Act::Run);
        m.step(&[press(true, false, true)]);
        assert_eq!(m.players[0].body.act, Act::Jump);
        // 放開方向就停下來
        let entrants = [Entrant { slot: 0, character: "aming" }];
        let mut m = Match::new(&entrants, 0, Difficulty::Easy, 3, 1);
        while m.phase != Phase::Play {
            m.step(&[Input::default()]);
        }
        m.step(&[press(true, false, false)]);
        m.step(&[Input::default()]);
        m.step(&[press(true, false, false)]);
        assert_eq!(m.players[0].body.act, Act::Run);
        m.step(&[Input::default()]);
        assert_eq!(m.players[0].body.act, Act::Stand);
    }

    /// 暫停選單「從本關開頭」：回到進這一關時的命、體力、分數，位置回到起點
    #[test]
    fn restart_stage_restores_start() {
        let entrants = [Entrant { slot: 0, character: "xiaocui" }];
        let mut m = Match::new(&entrants, 0, Difficulty::Easy, 3, 1);
        while m.phase != Phase::Play {
            m.step(&[Input::default()]);
        }
        let (lives, hp, score, x) = (m.players[0].lives, m.players[0].body.hp, m.players[0].score, m.players[0].body.x);
        m.players[0].lives = 1;
        m.players[0].body.hp = 5;
        m.players[0].score = 9999;
        m.players[0].body.x += 300.0;
        m.restart_stage();
        assert_eq!((m.players[0].lives, m.players[0].body.hp, m.players[0].score), (lives, hp, score));
        assert_eq!(m.players[0].body.x, x);
        assert_eq!((m.world, m.sub, m.phase), (0, 0, Phase::Intro));
    }

    /// 武器：站在旁邊按攻擊撿起來、揮了用武器打、被打倒會掉在地上、跑步中按攻擊丟出去
    #[test]
    fn weapons_pick_swing_drop_throw() {
        use crate::game::ItemKind;
        let entrants = [Entrant { slot: 0, character: "aming" }];
        let mut m = Match::new(&entrants, 0, Difficulty::Easy, 3, 1);
        while m.phase != Phase::Play {
            m.step(&[Input::default()]);
        }
        let (x, z) = (m.players[0].body.x, m.players[0].body.z);
        m.spawn_item(ItemKind::Bat, x + 20.0, z);
        let atk = Input { attack_pressed: true, ..Input::default() };
        m.step(&[atk]);
        assert_eq!(m.players[0].weapon, Some(ItemKind::Bat), "撿起來");
        assert!(!m.items.iter().any(|it| it.kind == ItemKind::Bat), "地上的拿掉了");
        for _ in 0..12 {
            m.step(&[Input::default()]);
        }
        m.step(&[atk]);
        assert_eq!(m.players[0].body.mv, Move::Bat, "拿著武器按攻擊是用武器打");
        // 被打倒：武器掉在地上
        for _ in 0..30 {
            m.step(&[Input::default()]);
        }
        // （遊戲裡是在「玩家移動之後」被打倒；這裡在移動之前設，所以 t 從 -1 開始，移動時會加到 0）
        m.players[0].body.act = Act::Fall;
        m.players[0].body.t = -1;
        m.players[0].body.y = 1.0;
        m.players[0].body.vy = 5.0;
        m.step(&[Input::default()]);
        assert_eq!(m.players[0].weapon, None);
        assert!(m.items.iter().any(|it| it.kind == ItemKind::Bat), "掉在地上");
        // 跑步中按攻擊：丟出去
        let mut m = Match::new(&entrants, 0, Difficulty::Easy, 3, 1);
        while m.phase != Phase::Play {
            m.step(&[Input::default()]);
        }
        m.players[0].weapon = Some(ItemKind::Blade);
        m.players[0].weapon_uses = 5;
        m.step(&[press(true, false, false)]);
        m.step(&[Input::default()]);
        m.step(&[press(true, false, false)]);
        assert_eq!(m.players[0].body.act, Act::Run);
        m.step(&[press(true, true, false)]);
        assert_eq!(m.players[0].weapon, None);
        assert_eq!(m.shots.len(), 1, "丟出去變成飛行道具");
    }

    /// 必殺要集滿氣才能放，放完氣就歸零（不能一直放）
    #[test]
    fn special_needs_full_gauge() {
        let entrants = [Entrant { slot: 0, character: "xiaocui" }];
        let mut m = Match::new(&entrants, 0, Difficulty::Easy, 3, 1);
        while m.phase != Phase::Play {
            m.step(&[Input::default()]);
        }
        let sp = Input { special_pressed: true, ..Input::default() };
        assert!(m.players[0].sp < crate::game::SP_MAX);
        m.step(&[sp]);
        assert_ne!(m.players[0].body.mv, Move::Special, "氣不夠不能放");
        m.players[0].sp = crate::game::SP_MAX;
        m.step(&[Input::default()]);
        m.step(&[sp]);
        assert_eq!(m.players[0].body.mv, Move::Special);
        assert_eq!(m.players[0].sp, 0);
    }
}
