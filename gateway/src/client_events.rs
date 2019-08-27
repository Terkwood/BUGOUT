use serde_derive::{Deserialize, Serialize};

use crate::compact_ids::CompactId;
use crate::model::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ClientEvents {
    MoveMade(MoveMadeEvent),
    MoveRejected(MoveRejectedEvent),
    Reconnected(ReconnectedEvent),
    HistoryProvided(HistoryProvidedEvent),
    GameReady(GameReadyClientEvent),
    PrivateGameRejected(PrivateGameRejectedClientEvent),
    WaitForOpponent(WaitForOpponentClientEvent),
}

impl ClientEvents {
    pub fn game_id(&self) -> Option<GameId> {
        match self {
            ClientEvents::MoveMade(e) => Some(e.game_id),
            ClientEvents::MoveRejected(e) => Some(e.game_id),
            ClientEvents::Reconnected(e) => Some(e.game_id),
            ClientEvents::HistoryProvided(e) => Some(e.game_id),
            ClientEvents::GameReady(e) => Some(e.game_id),
            ClientEvents::WaitForOpponent(w) => Some(w.game_id),
            _ => None, // TODO priv game rejected
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaitForOpponentClientEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrivateGameRejectedClientEvent {
    #[serde(rename = "gameId")]
    pub game_id: CompactId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameReadyClientEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
}
