use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
pub struct GameId(pub Uuid);

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
pub struct ReqId(pub Uuid);

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
pub struct EventId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize, Eq)]
pub enum Player {
    BLACK,
    WHITE,
}

impl Player {
    pub fn from_str(s: &str) -> Player {
        let mut trimmed = s.trim().to_ascii_lowercase();
        if trimmed.pop() == Some('w') {
            Player::WHITE
        } else {
            Player::BLACK
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Player::BLACK => "BLACK".to_string(),
            Player::WHITE => "WHITE".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct MakeMoveCommand {
    pub game_id: GameId,
    pub req_id: ReqId,
    pub player: Player,
    pub coord: Option<Coord>,
}

#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash, Serialize, Deserialize)]
pub struct Coord {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct GameState {
    pub board: Board,
    pub captures: Captures,
    pub turn: u16,
    pub player_up: Player,
    pub moves: Vec<MoveMade>,
}
impl Default for GameState {
    fn default() -> Self {
        GameState {
            board: Board::default(),
            captures: Captures::default(),
            turn: 1,
            player_up: Player::BLACK,
            moves: vec![],
        }
    }
}

const DEFAULT_BOARD_SIZE: u16 = 19;
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Board {
    pub pieces: HashMap<Coord, Player>,
    pub size: u16,
}
impl Default for Board {
    fn default() -> Self {
        Board {
            pieces: HashMap::new(),
            size: DEFAULT_BOARD_SIZE,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Captures {
    pub black: u16,
    pub white: u16,
}
impl Default for Captures {
    fn default() -> Self {
        Captures { black: 0, white: 0 }
    }
}

// TODO defaults
/// An event signalling the acceptance of a move.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoveMade {
    pub game_id: GameId,
    pub reply_to: ReqId,
    pub event_id: EventId,
    pub player: Player,
    pub coord: Option<Coord>,
    pub captured: Vec<Coord>,
}
