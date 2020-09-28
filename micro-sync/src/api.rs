use crate::model::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct ReqSync {
    pub session_id: SessionId,
    pub req_id: ReqId,
    pub game_id: GameId,
    pub player_up: Player,
    pub turn: u32,
    pub last_move: Option<Move>,
}

#[derive(Clone, Debug, Serialize)]
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

//// This command requests that a move be judged for
/// pubidity and communicated to all other participants.
/// Emitted by micro-sync in the case that the backend
/// needs to catch up with the client's view of their
/// own state.
#[derive(Clone, Debug, Serialize)]
pub struct MakeMove {
    pub game_id: GameId,
    pub req_id: ReqId,
    pub player: Player,
    pub coord: Option<Coord>,
}

/// An event signalling the acceptance of a move.
/// This is emitted by changelog service.
#[derive(Clone, Deserialize, Debug)]
pub struct MoveMade {
    pub game_id: GameId,
    pub reply_to: ReqId,
    pub event_id: EventId,
    pub player: Player,
    pub coord: Option<Coord>,
    pub captured: Vec<Coord>,
}
