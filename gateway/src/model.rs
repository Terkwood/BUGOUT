use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

use crate::compact_ids::CompactId;

pub type GameId = Uuid;
pub type ReqId = Uuid;
pub type EventId = Uuid;
pub type ClientId = Uuid;

pub const DEFAULT_BOARD_SIZE: usize = 19;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Coord {
    pub x: u16,
    pub y: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Player {
    BLACK,
    WHITE,
}

impl Player {
    pub fn other(&self) -> Player {
        match self {
            Player::BLACK => Player::WHITE,
            Player::WHITE => Player::BLACK,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MakeMoveCommand {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "reqId")]
    pub req_id: ReqId,
    pub player: Player,
    pub coord: Option<Coord>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RequestGameIdCommand {
    #[serde(rename = "reqId")]
    pub req_id: ReqId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProvideHistoryCommand {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "reqId")]
    pub req_id: ReqId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReconnectCommand {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "reqId")]
    pub req_id: ReqId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JoinPrivateGameClientCommand {
    #[serde(rename = "gameId")]
    pub game_id: CompactId,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum ClientCommands {
    MakeMove(MakeMoveCommand),
    Beep,
    RequestOpenGame(RequestGameIdCommand),
    Reconnect(ReconnectCommand),
    ProvideHistory(ProvideHistoryCommand),
    JoinPrivateGame(JoinPrivateGameClientCommand),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinPrivateGameKafkaCommand {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
}

// https://github.com/Terkwood/BUGOUT/issues/81
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum KafkaCommands {
    MakeMove(MakeMoveCommand),
    ProvideHistory(ProvideHistoryCommand),
    JoinPrivateGame(JoinPrivateGameKafkaCommand),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MoveMadeEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "replyTo")]
    pub reply_to: ReqId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
    pub player: Player,
    pub coord: Option<Coord>,
    pub captured: Vec<Coord>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MoveRejectedEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "replyTo")]
    pub reply_to: ReqId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
    pub player: Player,
    pub coord: Coord,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenGameReplyEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "replyTo")]
    pub reply_to: ReqId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReconnectedEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "replyTo")]
    pub reply_to: ReqId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
    #[serde(rename = "playerUp")]
    pub player_up: Player,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoryProvidedEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "replyTo")]
    pub reply_to: ReqId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
    pub moves: Vec<Move>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Events {
    MoveMade(MoveMadeEvent),
    MoveRejected(MoveRejectedEvent),
    OpenGameReply(OpenGameReplyEvent),
    Reconnected(ReconnectedEvent),
    HistoryProvided(HistoryProvidedEvent),
}

impl Events {
    pub fn game_id(&self) -> GameId {
        match self {
            Events::MoveMade(e) => e.game_id,
            Events::MoveRejected(e) => e.game_id,
            Events::OpenGameReply(e) => e.game_id,
            Events::Reconnected(e) => e.game_id,
            Events::HistoryProvided(e) => e.game_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Captures {
    pub black: u16,
    pub white: u16,
}

impl Default for Captures {
    fn default() -> Captures {
        Captures { black: 0, white: 0 }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Move {
    player: Player,
    coord: Option<Coord>,
    turn: i32,
}

#[cfg(test)]
mod tests {
    use crate::compact_ids::CompactId;
    use crate::model::*;
    use uuid::Uuid;

    #[test]
    fn serialize_move_command() {
        let game_id = Uuid::new_v4();
        let req_id = Uuid::new_v4();

        assert_eq!(
            serde_json::to_string(&super::ClientCommands::MakeMove (super::MakeMoveCommand{
                game_id,
                req_id,
                player: super::Player::BLACK,
                coord: Some(super::Coord { x: 0, y: 0 })
            }))
            .unwrap(),
            format!("{{\"type\":\"MakeMove\",\"gameId\":\"{:?}\",\"reqId\":\"{:?}\",\"player\":\"BLACK\",\"coord\":{{\"x\":0,\"y\":0}}}}", game_id, req_id)
        )
    }

    #[test]
    fn deserialize_join_priv_game_client_command() {
        let compact_game_id = CompactId::encode(Uuid::new_v4());

        let json = &format!(
            "{{\"type\":\"JoinPrivateGame\",\"gameId\":\"{}\"}}",
            compact_game_id.0
        );

        let d: ClientCommands = serde_json::from_str(json).unwrap();

        assert_eq!(
            d,
            ClientCommands::JoinPrivateGame(JoinPrivateGameClientCommand {
                game_id: compact_game_id
            })
        )
    }
}
