//! 連線訊息（JSON）的處理：hello、input、host 指令…，回覆 welcome、error、room、frame。
//! 區網伺服器（src-tauri/src/server.rs，WebSocket）與線上房間的房主瀏覽器（crates/quest-wasm，WebRTC）
//! 共用這一份，所以兩種房間的行為一模一樣。

use crate::ai::Difficulty;
use crate::game::{Event, MAX_PLAYERS};
use crate::room::{Room, Settings};
use serde_json::{json, Value};

/// 一條連線（一個玩家或觀眾）
#[derive(Default)]
pub struct Conn {
    pub seat: Option<usize>,
    pub is_host: bool,
    /// 送過 hello 了（之後才收得到大廳資訊與影格）
    pub hello: bool,
}

pub fn err(msg: impl Into<String>) -> Value {
    json!({"type": "error", "message": msg.into()})
}

fn welcome(conn: &Conn, token: Option<&str>) -> Value {
    json!({"type": "welcome", "seat": conn.seat, "token": token, "is_host": conn.is_host})
}

/// 給這條連線的大廳資訊
pub fn room_message(room: &Room, conn: &Conn) -> Value {
    let mut view = room.view(conn.seat);
    view["is_host"] = json!(conn.is_host);
    json!({"type": "room", "state": view})
}

/// 比賽影格的訊息
pub fn frame_message(state: &Value) -> Value {
    json!({"type": "frame", "state": state})
}

/// 這一格有過關／全軍覆沒／破關（大廳資訊要更新：聊天多了一行系統訊息）
pub fn just_ended(room: &Room) -> bool {
    room.game
        .as_ref()
        .is_some_and(|g| g.events.iter().any(|e| matches!(e, Event::Over | Event::Clear { .. } | Event::Ending)))
}

/// 連線中斷：比賽中由電腦代打
pub fn drop_conn(room: &mut Room, conn: &mut Conn) -> bool {
    match conn.seat.take() {
        Some(s) => {
            room.disconnect(s);
            true
        }
        None => false,
    }
}

/// 處理一則訊息，回傳（要回給這條連線的訊息, 大廳資訊有沒有變）
pub fn handle(room: &mut Room, host_key: &str, conn: &mut Conn, v: &Value) -> (Vec<Value>, bool) {
    let ty = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
    let str_of = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("");
    match ty {
        "hello" => {
            conn.hello = true;
            if let Some(k) = v.get("host_key").and_then(|x| x.as_str()) {
                conn.is_host = !k.is_empty() && k == host_key;
            }
            if let Some(s) = conn.seat.take() {
                room.disconnect(s);
            }
            // 帶著 token 回來：拿回原本的座位
            let token = str_of("token");
            if let Some(s) = room.seat_of_token(token) {
                conn.seat = Some(s);
                room.connect(s);
                if conn.is_host {
                    room.host_seat = Some(s);
                }
                return (vec![welcome(conn, Some(token))], true);
            }
            if v.get("spectate").and_then(|x| x.as_bool()).unwrap_or(false) {
                return (vec![welcome(conn, None)], false);
            }
            sit(room, conn, str_of("name"), v.get("character").and_then(|x| x.as_str()))
        }
        // 觀眾在大廳按「入席」
        "sit" => {
            if conn.seat.is_some() {
                return (vec![], false);
            }
            sit(room, conn, str_of("name"), v.get("character").and_then(|x| x.as_str()))
        }
        "input" => {
            if let Some(s) = conn.seat {
                let bits = v.get("b").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                let seq = v.get("s").and_then(|x| x.as_u64()).unwrap_or(0);
                room.set_input(s, bits, seq);
            }
            (vec![], false)
        }
        "character" => {
            let Some(s) = conn.seat else { return (vec![err("你沒有座位")], false) };
            match room.set_character(s, str_of("id")) {
                Ok(()) => (vec![], true),
                Err(e) => (vec![err(e)], false),
            }
        }
        "chat" => {
            room.push_chat(conn.seat, str_of("text"));
            (vec![], true)
        }
        "leave" => {
            if let Some(s) = conn.seat.take() {
                let _ = room.remove_seat(s);
            }
            (vec![welcome(conn, None)], true)
        }
        "host" => {
            if !conn.is_host {
                return (vec![err("只有房主可以這樣做")], false);
            }
            let seat = v.get("seat").and_then(|x| x.as_u64()).map(|s| s as usize).filter(|s| *s < MAX_PLAYERS);
            let result = match str_of("cmd") {
                "add_bot" => {
                    let d: Difficulty = v.get("difficulty").and_then(|d| serde_json::from_value(d.clone()).ok()).unwrap_or_default();
                    room.add_bot(seat, d).map(|_| ())
                }
                "remove" => match seat {
                    Some(s) if Some(s) != room.host_seat => room.remove_seat(s),
                    _ => Err("不能移除這個座位".into()),
                },
                "settings" => match v.get("settings").and_then(|s| serde_json::from_value::<Settings>(s.clone()).ok()) {
                    Some(s) => {
                        room.update_settings(s);
                        Ok(())
                    }
                    None => Err("設定格式不對".into()),
                },
                "start" => room.start(),
                "continue" => room.continue_game(),
                "reset" => {
                    room.reset_to_lobby();
                    Ok(())
                }
                _ => Err("不認得的指令".into()),
            };
            match result {
                Ok(()) => (vec![], true),
                Err(e) => (vec![err(e)], false),
            }
        }
        _ => (vec![], false),
    }
}

fn sit(room: &mut Room, conn: &mut Conn, name: &str, character: Option<&str>) -> (Vec<Value>, bool) {
    match room.join(name, character) {
        Ok((s, token)) => {
            conn.seat = Some(s);
            room.connect(s);
            if conn.is_host {
                room.host_seat = Some(s);
            }
            (vec![welcome(conn, Some(&token))], true)
        }
        Err(e) => (vec![err(format!("{}，先以觀眾身分進入", e)), welcome(conn, None)], true),
    }
}
