use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Serialize)]
pub enum Color {
    Black,
    White,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum ColorPref {
    Black,
    White,
    Any,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionColorPref {
    pub game_id: GameId,
    pub session_id: SessionId,
    pub color_pref: ColorPref,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameColorPref {
    Empty,
    Partial(SessionColorPref),
    Complete(SessionColorPref, SessionColorPref),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClientId(pub Uuid);
#[derive(Clone, Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct SessionId(pub Uuid);
#[derive(Clone, Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct GameId(pub Uuid);
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EventId(pub Uuid);

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionGame {
    pub session_id: SessionId,
    pub game_id: GameId,
}

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
