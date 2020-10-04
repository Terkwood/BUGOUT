pub mod api;

use core_model::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ColorPref {
    Black,
    White,
    Any,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SessionColorPref {
    pub session_id: SessionId,
    pub color_pref: ColorPref,
    pub client_id: ClientId,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GameColorPref {
    NotReady,
    Partial {
        game_id: GameId,
        pref: SessionColorPref,
    },
    Complete {
        game_id: GameId,
        prefs: (SessionColorPref, SessionColorPref),
    },
}
