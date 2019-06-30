use crate::model::{Captures, Player};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

/// This JSON object contains a complex
/// key for its board field.
/// ```json
/// {"board":{"pieces":{"0,0": "WHITE"},"size":19},"captures":{"black":0,"white":0},"turn":1,"playerUp":"BLACK"}
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct BoardJson {
    pub pieces: HashMap<String, Player>,
    pub size: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameStateJson {
    pub board: BoardJson,
    pub captures: Captures,
    pub turn: u32,
    #[serde(rename = "playerUp")]
    pub player_up: Player,
}
