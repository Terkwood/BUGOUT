use crate::model::*;
use serde_derive::{Deserialize, Serialize};
pub struct ChooseColorPref {}

pub struct ColorsChosen {
    pub game_id: GameId,
    pub black: ClientId,
    pub white: ClientId,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GameReady {
    pub game_id: GameId,
    pub sessions: (SessionId, SessionId),
    pub event_id: EventId,
}
