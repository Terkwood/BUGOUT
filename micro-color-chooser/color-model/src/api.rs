use super::*;
use core_model::*;
use serde_derive::{Deserialize, Serialize};

/// This service listens for the same game ready
/// event that is announced by game lobby
pub use lobby_model::api::GameReady;

#[derive(Clone, Debug, Deserialize)]
pub struct ChooseColorPref {
    pub client_id: ClientId,
    pub color_pref: ColorPref,
    pub session_id: SessionId,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ColorsChosen {
    pub game_id: GameId,
    pub black: ClientId,
    pub white: ClientId,
}
