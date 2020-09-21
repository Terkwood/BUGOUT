use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Clone, Serialize, Deserialize)]
pub struct GameId(Uuid);
pub struct ReqId(Uuid);
pub struct EventId(Uuid);

pub enum Player {
    BLACK,
    WHITE,
}

pub struct Coord {
    pub x: u16,
    pub y: u16,
}

pub struct MoveEvent {
    pub player: Player,
    coord: Option<Coord>,
    turn: u32,
}

pub struct GameState {
    pub moves: Option<Vec<MoveEvent>>,
    pub player_up: Player,
}
