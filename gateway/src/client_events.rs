use serde_derive::{Deserialize, Serialize};

use crate::compact_ids::CompactId;
use crate::env::*;
use crate::idle_status::IdleStatus;
use crate::model::*;
use core_model::*;
use move_model::Player;

/// Events which will be sent to the browser
/// from gateway
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
    IdentityAcknowledged(Identity),
    OpponentQuit,
    BotAttached(micro_model_bot::gateway::BotAttached),
    SyncReply(SyncReplyClientEvent),
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
            ClientEvents::BotAttached(b) => Some(GameId(b.game_id.0)),
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
            CompactId::encode(game_id.0).0
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
    #[serde(rename = "boardSize")]
    pub board_size: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YourColorEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "yourColor")]
    pub your_color: Player,
}

/// This event announces the data-layer view
/// of the game, including player, turn, and
/// moves made.  The reply_to field is used
/// to indicate which ReqSync command triggered
/// this SyncReply.  Browser is advised
/// to discard any SyncReplies that are
/// not tied to its most recent ReqSync.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncReplyClientEvent {
    pub reply_to: ReqId,
    pub player_up: Player,
    pub turn: u32,
    pub moves: Vec<Move>,
}
