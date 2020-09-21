use crate::model::*;
pub struct ProvideHistory {
    pub game_id: GameId,
    pub req_id: ReqId,
}

pub struct HistoryProvided {
    pub game_id: GameId,
    pub reply_to: ReqId,
    pub event_id: EventId,
    pub moves: Vec<Move>,
    pub epoch_millis: u64,
}
