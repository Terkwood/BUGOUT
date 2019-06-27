use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Coord {
    x: u16,
    y: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Player {
    BLACK,
    WHITE,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum MoveEvents {
    MoveMade {
        game_id: Uuid,
        reply_to: Uuid,
        event_id: Uuid,
        player: Player,
    },
    MoveRejected {
        reply_to: Uuid,
    },
}
