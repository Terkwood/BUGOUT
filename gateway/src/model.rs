use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

pub type GameId = Uuid;
pub type ReqId = Uuid;
pub type EventId = Uuid;
pub type ClientId = Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
pub struct GameClients {
    pub first: ClientId,
    pub second: ClientId,
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
