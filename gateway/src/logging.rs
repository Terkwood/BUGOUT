use rand::seq::SliceRandom;

use crate::websocket::WsSession;
use move_model::Player;

pub fn emoji(player: &Player) -> String {
    match player {
        Player::BLACK => vec!["♚", "♛", "♜", "♝", "♞", "♟"]
            .choose(&mut rand::thread_rng())
            .map(|s| s.to_string())
            .unwrap_or("♚".to_owned()),
        Player::WHITE => vec!["♔", "♕", "♖", "♗", "♘", "♙"]
            .choose(&mut rand::thread_rng())
            .map(|s| s.to_string())
            .unwrap_or("♔".to_owned()),
    }
}

pub fn session_code(ws_session: &WsSession) -> String {
    format!(
        "{} {}",
        ws_session
            .client_id
            .map(|cid| crate::short_uuid(cid.0))
            .unwrap_or(crate::EMPTY_SHORT_UUID.to_string()),
        ws_session
            .current_game
            .map(|gid| crate::short_uuid(gid.0))
            .unwrap_or(crate::EMPTY_SHORT_UUID.to_string())
    )
    .to_string()
}
