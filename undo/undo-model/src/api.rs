use core_model::GameId;
use move_model::{GameState, Player};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UndoMove {
    pub game_id: GameId,
    pub player: Player,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MoveUndone {
    pub game_id: GameId,
    pub player: Player,
    pub game_state: GameState,
}
