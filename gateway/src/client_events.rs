use serde_derive::{Deserialize, Serialize};

use crate::compact_ids::CompactId;
use crate::env::*;
use crate::idle_status::IdleStatus;
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
    YourColor(YourColorEvent),
    IdleStatusProvided(IdleStatus),
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
            ClientEvents::YourColor(y) => Some(y.game_id),
            _ => None, // priv game rejected, see https://github.com/Terkwood/BUGOUT/issues/90
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Link(String);
impl Link {
    pub fn new(game_id: GameId) -> Link {
        Link(format!(
            "{}/?join={}",
            LINK_TO.to_string(),
            CompactId::encode(game_id).0
        ))
    }
}

/// If it's private visibility, you'll see a link
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaitForOpponentClientEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
    pub visibility: Visibility,
    pub link: Option<Link>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YourColorEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "yourColor")]
    pub your_color: Player,
}
