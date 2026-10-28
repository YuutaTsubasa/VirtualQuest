//! 連線訊息（JSON）的處理：hello、input、host 指令…，回覆 welcome、error、room、frame。
//! 區網伺服器（VirtualCore 的 virtual-lan，WebSocket）與線上房間的房主瀏覽器（crates/quest-wasm，WebRTC）
//! 共用這一份，所以兩種房間的行為一模一樣。程式在 VirtualCore 的 virtual-room（五款共用），
//! 闖天下自己的部分（輸入、出發、接關、每格前進）在 room.rs 的 `RoomGame` 實作。

pub use virtual_room::protocol::{drop_conn, err, frame_message, handle, room_message, Conn};
