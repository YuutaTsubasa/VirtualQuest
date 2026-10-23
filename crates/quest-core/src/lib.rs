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
