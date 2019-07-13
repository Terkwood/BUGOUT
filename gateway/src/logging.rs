use rand::seq::SliceRandom;
use uuid::Uuid;

use crate::model::Player;

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

pub fn short_uuid(uuid: Uuid) -> String {
    uuid.to_string()[..8].to_string()
}

pub fn short_time() -> i64 {
    time::now_utc().to_timespec().sec % 10_000
}
