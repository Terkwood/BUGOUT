use rand::seq::SliceRandom;

use crate::model::Player;
use crate::websocket::WsSession;

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
        crate::short_uuid(ws_session.client_id),
        ws_session
            .current_game
            .map(|gid| crate::short_uuid(gid))
            .unwrap_or(crate::EMPTY_SHORT_UUID.to_string())
    )
    .to_string()
}
