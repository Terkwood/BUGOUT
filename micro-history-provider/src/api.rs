use crate::model::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct ProvideHistory {
    pub game_id: GameId,
    pub req_id: ReqId,
}

#[derive(Clone, Debug, Serialize)]
pub struct HistoryProvided {
    pub game_id: GameId,
    pub reply_to: ReqId,
    pub event_id: EventId,
    pub moves: Vec<Move>,
    pub epoch_millis: u64,
}
