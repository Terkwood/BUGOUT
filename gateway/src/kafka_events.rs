use std::time::SystemTime;

use serde_derive::{Deserialize, Serialize};

use crate::client_events::*;
use crate::compact_ids::CompactId;
use crate::model::*;

#[derive(Debug)]
pub enum KafkaEvents {
    MoveMade(MoveMadeEvent),
    MoveRejected(MoveRejectedEvent),
    HistoryProvided(HistoryProvidedEvent),
    GameReady(GameReadyKafkaEvent),
    PrivateGameRejected(PrivateGameRejectedKafkaEvent),
    WaitForOpponent(WaitForOpponentKafkaEvent),
    ColorsChosen(ColorsChosenEvent),
    Shutdown(SystemTime),
}

impl KafkaEvents {
    pub fn to_client_event(self) -> Option<ClientEvents> {
        match self {
            KafkaEvents::MoveMade(m) => Some(ClientEvents::MoveMade(m)),
            KafkaEvents::MoveRejected(m) => Some(ClientEvents::MoveRejected(m)),
            KafkaEvents::HistoryProvided(h) => Some(ClientEvents::HistoryProvided(h)),
            // Dummy impl, don't trust it
            KafkaEvents::ColorsChosen(c) => Some(ClientEvents::YourColor(YourColorEvent {
                game_id: c.game_id,
                your_color: Player::BLACK,
            })),
            KafkaEvents::GameReady(g) => Some(ClientEvents::GameReady(GameReadyClientEvent {
                game_id: g.game_id,
                event_id: g.event_id,
            })),
            KafkaEvents::PrivateGameRejected(p) => Some(ClientEvents::PrivateGameRejected(
                PrivateGameRejectedClientEvent {
                    game_id: CompactId::encode(p.game_id),
                    event_id: p.event_id,
                },
            )),

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
                Some(ClientEvents::WaitForOpponent(WaitForOpponentClientEvent {
                    game_id,
                    event_id,
                    visibility,
                    link,
                }))
            }
            KafkaEvents::Shutdown(_) => None,
        }
    }

    pub fn game_id(&self) -> Option<GameId> {
        match self {
            KafkaEvents::MoveMade(e) => Some(e.game_id),
            KafkaEvents::MoveRejected(e) => Some(e.game_id),
            KafkaEvents::HistoryProvided(e) => Some(e.game_id),
            KafkaEvents::GameReady(e) => Some(e.game_id),
            KafkaEvents::PrivateGameRejected(e) => Some(e.game_id),
            KafkaEvents::WaitForOpponent(e) => Some(e.game_id),
            KafkaEvents::ColorsChosen(e) => Some(e.game_id),
            KafkaEvents::Shutdown(_) => None,
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
