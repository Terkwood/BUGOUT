use serde_derive::{Deserialize, Serialize};

use crate::client_events::*;
use crate::compact_ids::CompactId;
use crate::model::*;

pub enum KafkaEvents {
    MoveMade(MoveMadeEvent),
    MoveRejected(MoveRejectedEvent),
    HistoryProvided(HistoryProvidedEvent),
    GameReady(GameReadyKafkaEvent),
    PrivateGameRejected(PrivateGameRejectedKafkaEvent),
    WaitForOpponent(WaitForOpponentKafkaEvent),
}

impl KafkaEvents {
    pub fn to_client_event(self) -> ClientEvents {
        match self {
            KafkaEvents::MoveMade(m) => ClientEvents::MoveMade(m),
            KafkaEvents::MoveRejected(m) => ClientEvents::MoveRejected(m),
            KafkaEvents::HistoryProvided(h) => ClientEvents::HistoryProvided(h),
            KafkaEvents::GameReady(g) => ClientEvents::GameReady(GameReadyClientEvent {
                game_id: g.game_id,
                event_id: g.event_id,
            }),
            KafkaEvents::PrivateGameRejected(p) => {
                ClientEvents::PrivateGameRejected(PrivateGameRejectedClientEvent {
                    game_id: CompactId::encode(p.game_id),
                    event_id: p.event_id,
                })
            }
            KafkaEvents::WaitForOpponent(WaitForOpponentKafkaEvent {
                game_id,
                client_id: _,
                event_id,
                visibility,
            }) => {
                let link = match visibility {
                    Visibility::Public => None,
                    Visibility::Private => Some(Link::new(game_id)),
                };
                ClientEvents::WaitForOpponent(WaitForOpponentClientEvent {
                    game_id,
                    event_id,
                    visibility,
                    link,
                })
            }
        }
    }

    pub fn game_id(&self) -> GameId {
        match self {
            KafkaEvents::MoveMade(e) => e.game_id,
            KafkaEvents::MoveRejected(e) => e.game_id,
            KafkaEvents::HistoryProvided(e) => e.game_id,
            KafkaEvents::GameReady(e) => e.game_id,
            KafkaEvents::PrivateGameRejected(e) => e.game_id,
            KafkaEvents::WaitForOpponent(e) => e.game_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameReadyKafkaEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    pub clients: GameClients,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaitForOpponentKafkaEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
    pub visibility: Visibility,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrivateGameRejectedKafkaEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
}
