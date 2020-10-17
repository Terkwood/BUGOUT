use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

pub type GameId = Uuid;
pub type ReqId = Uuid;
pub type EventId = Uuid;
pub type ClientId = Uuid;
pub type SessionId = Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub struct Identity {
    #[serde(rename = "clientId")]
    pub client_id: ClientId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub struct Coord {
    pub x: u16,
    pub y: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum ColorPref {
    Black,
    White,
    Any,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ColorsChosenEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    pub black: ClientId,
    pub white: ClientId,
}

/// A request for a move to be made in a given game.
/// These moves are subsequently judged for correctness.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MakeMoveCommand {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "reqId")]
    pub req_id: ReqId,
    pub player: Player,
    pub coord: Option<Coord>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ProvideHistoryCommand {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "reqId")]
    pub req_id: ReqId,
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
pub struct GameSessions {
    pub first: SessionId,
    pub second: SessionId,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Captures {
    pub black: u16,
    pub white: u16,
}

impl Default for Captures {
    fn default() -> Captures {
        Captures { black: 0, white: 0 }
    }
}

/// The basic components of a move, used by several
/// other data structures
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Move {
    pub player: Player,
    pub coord: Option<Coord>,
    pub turn: i32,
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Player::BLACK => "B",
                Player::WHITE => "W",
            }
        )
    }
}
