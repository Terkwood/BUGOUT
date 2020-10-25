mod from;

use crate::client_events::*;
use crate::compact_ids::CompactId;
use crate::model::*;
use serde_derive::{Deserialize, Serialize};

/// Events produced by Redis Streams
#[derive(Debug)]
pub enum BackendEvents {
    MoveMade(MoveMadeEvent),
    MoveRejected(MoveRejectedEvent),
    HistoryProvided(HistoryProvidedEvent),
    GameReady(GameReadyBackendEvent),
    PrivateGameRejected(PrivateGameRejectedBackendEvent),
    WaitForOpponent(WaitForOpponentBackendEvent),
    ColorsChosen(ColorsChosenEvent),
    BotAttached(bot_model::api::BotAttached),
    SyncReply(SyncReplyBackendEvent),
}

impl BackendEvents {
    pub fn to_client_event(self) -> ClientEvents {
        match self {
            BackendEvents::MoveMade(m) => ClientEvents::MoveMade(m),
            BackendEvents::MoveRejected(m) => ClientEvents::MoveRejected(m),
            BackendEvents::HistoryProvided(h) => ClientEvents::HistoryProvided(h),
            // Dummy impl, don't trust it
            BackendEvents::ColorsChosen(c) => ClientEvents::YourColor(YourColorEvent {
                game_id: c.game_id,
                your_color: Player::BLACK,
            }),
            BackendEvents::GameReady(GameReadyBackendEvent {
                game_id,
                event_id,
                board_size,
                sessions: _,
            }) => ClientEvents::GameReady(GameReadyClientEvent {
                game_id,
                event_id,
                board_size,
            }),
            BackendEvents::PrivateGameRejected(p) => {
                ClientEvents::PrivateGameRejected(PrivateGameRejectedClientEvent {
                    game_id: CompactId::encode(p.game_id),
                    event_id: p.event_id,
                })
            }

            BackendEvents::WaitForOpponent(WaitForOpponentBackendEvent {
                game_id,
                session_id: _,
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

            BackendEvents::BotAttached(ba) => ClientEvents::BotAttached(ba),
            BackendEvents::SyncReply(SyncReplyBackendEvent {
                session_id: _,
                game_id: _,
                player_up,
                reply_to,
                turn,
                moves,
            }) => ClientEvents::SyncReply(SyncReplyClientEvent {
                player_up,
                turn,
                reply_to,
                moves,
            }),
        }
    }

    pub fn game_id(&self) -> GameId {
        match self {
            BackendEvents::MoveMade(e) => e.game_id,
            BackendEvents::MoveRejected(e) => e.game_id,
            BackendEvents::HistoryProvided(e) => e.game_id,
            BackendEvents::GameReady(e) => e.game_id,
            BackendEvents::PrivateGameRejected(e) => e.game_id,
            BackendEvents::WaitForOpponent(e) => e.game_id,
            BackendEvents::ColorsChosen(e) => e.game_id,
            BackendEvents::BotAttached(e) => e.game_id.0,
            BackendEvents::SyncReply(e) => e.game_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameReadyBackendEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    pub sessions: GameSessions,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
    #[serde(rename = "boardSize")]
    pub board_size: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaitForOpponentBackendEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "sessionId")]
    pub session_id: SessionId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
    pub visibility: Visibility,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrivateGameRejectedBackendEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
    #[serde(rename = "sessionId")]
    pub session_id: SessionId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncReplyBackendEvent {
    pub session_id: SessionId,
    pub reply_to: ReqId,
    pub player_up: Player,
    pub turn: u32,
    pub moves: Vec<Move>,
    pub game_id: GameId,
}
