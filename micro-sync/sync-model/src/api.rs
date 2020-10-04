use super::*;
use core_model::*;
use move_model::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReqSync {
    pub session_id: SessionId,
    pub req_id: ReqId,
    pub game_id: GameId,
    pub player_up: Player,
    pub turn: u32,
    pub last_move: Option<Move>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct SyncReply {
    pub session_id: SessionId,
    pub reply_to: ReqId,
    pub game_id: GameId,
    pub player_up: Player,
    pub turn: u32,
    pub moves: Vec<Move>,
}

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
