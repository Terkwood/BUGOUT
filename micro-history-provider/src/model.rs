use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct GameId(pub Uuid);
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ReqId(pub Uuid);
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EventId(pub Uuid);

#[derive(Clone, Serialize, Deserialize, Debug, Copy, PartialEq)]
pub enum Player {
    BLACK,
    WHITE,
}

#[derive(Clone, Serialize, Deserialize, Debug, Copy, PartialEq)]
pub struct Coord {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MoveEvent {
    pub player: Player,
    pub coord: Option<Coord>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Move {
    pub player: Player,
    pub coord: Option<Coord>,
    pub turn: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameState {
    pub moves: Option<Vec<MoveEvent>>,
    pub player_up: Player,
}
