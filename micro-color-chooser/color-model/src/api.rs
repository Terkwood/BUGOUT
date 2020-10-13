use super::*;
use core_model::*;
use serde_derive::{Deserialize, Serialize};

/// emitted by the game lobby
pub use lobby_model::api::GameReady;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChooseColorPref {
    pub client_id: ClientId,
    pub color_pref: ColorPref,
    pub session_id: SessionId,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ColorsChosen {
    pub game_id: GameId,
    pub black: ClientId,
    pub white: ClientId,
}
