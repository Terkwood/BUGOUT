extern crate bincode;

pub mod gateway;

use micro_model_moves::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeMove {
    pub game_id: GameId,
    pub game_state: GameState,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoveComputed {
    pub game_id: GameId,
    pub player: Player,
    pub alphanum_coord: Option<AlphaNumCoord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlphaNumCoord(pub char, pub u16);
