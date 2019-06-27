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
pub enum Commands {
    MakeMove {
        game_id: Uuid,
        request_id: Uuid,
        player: Player,
        coord: Option<Coord>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Events {
    MoveMade {
        game_id: Uuid,
        reply_to: Uuid,
        event_id: Uuid,
        player: Player,
        coord: Option<Coord>,
        captured: Vec<Coord>,
    },
    MoveRejected {
        game_id: Uuid,
        reply_to: Uuid,
        event_id: Uuid,
        player: Player,
        coord: Coord,
    },
}
