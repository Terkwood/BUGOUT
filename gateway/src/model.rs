use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

pub type GameId = Uuid;
pub type ReqId = Uuid;
pub type EventId = Uuid;
pub type ClientId = Uuid;

pub const DEFAULT_BOARD_SIZE: usize = 19;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Coord {
    pub x: u16,
    pub y: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Player {
    BLACK,
    WHITE,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MakeMoveCommand {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "reqId")]
    pub req_id: ReqId,
    pub player: Player,
    pub coord: Option<Coord>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestGameIdCommand {
    #[serde(rename = "reqId")]
    pub req_id: ReqId,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Commands {
    MakeMove(MakeMoveCommand),
    Beep,
    RequestGameId(RequestGameIdCommand),
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
pub struct GameIdReplyEvent {
    #[serde(rename = "gameId")]
    pub game_id: GameId,
    #[serde(rename = "replyTo")]
    pub reply_to: ReqId,
    #[serde(rename = "eventId")]
    pub event_id: EventId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Events {
    MoveMade(MoveMadeEvent),
    MoveRejected(MoveRejectedEvent),
    GameIdReply(GameIdReplyEvent),
}

impl Events {
    pub fn game_id(&self) -> GameId {
        match self {
            Events::MoveMade(e) => e.game_id,
            Events::MoveRejected(e) => e.game_id,
            Events::GameIdReply(e) => e.game_id,
        }
    }
}

pub enum BugoutMessage {
    Command { client_id: Uuid, command: Commands },
    Event(Events),
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

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    #[test]
    fn serialize_move_command() {
        let game_id = Uuid::new_v4();
        let req_id = Uuid::new_v4();

        assert_eq!(
            serde_json::to_string(&super::Commands::MakeMove (super::MakeMoveCommand{
                game_id,
                req_id,
                player: super::Player::BLACK,
                coord: Some(super::Coord { x: 0, y: 0 })
            }))
            .unwrap(),
            format!("{{\"type\":\"MakeMove\",\"gameId\":\"{:?}\",\"reqId\":\"{:?}\",\"player\":\"BLACK\",\"coord\":{{\"x\":0,\"y\":0}}}}", game_id, req_id)
        )
    }
}
