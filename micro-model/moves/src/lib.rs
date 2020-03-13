use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
pub struct GameId(pub Uuid);

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
pub struct ReqId(pub Uuid);

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
pub struct EventId(pub Uuid);
impl EventId {
    pub fn new() -> Self {
        EventId(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize, Eq, Hash)]
pub enum Player {
    BLACK,
    WHITE,
}

impl Player {
    pub fn from_str(s: &str) -> Player {
        let trimmed = s.trim().to_ascii_lowercase();
        if trimmed.chars().next() == Some('w') {
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

#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
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
impl Coord {
    pub fn of(x: u16, y: u16) -> Self {
        Coord { x, y }
    }
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

impl GameState {
    pub fn from(bytes: &[u8]) -> Result<GameState, std::boxed::Box<bincode::ErrorKind>> {
        bincode::deserialize(bytes)
    }
    pub fn serialize(&self) -> Result<Vec<u8>, std::boxed::Box<bincode::ErrorKind>> {
        Ok(bincode::serialize(&self)?)
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
impl MoveMade {
    pub fn serialize(&self) -> Result<Vec<u8>, std::boxed::Box<bincode::ErrorKind>> {
        Ok(bincode::serialize(&self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_game_state_ser_basic() {
        let gs = GameState::default();
        let result = gs.serialize();
        assert!(result.is_ok());
        assert!(result.unwrap().len() > 0)
    }

    #[test]
    fn test_game_state_serde_roundtrip() {
        let mut gs = GameState::default();
        gs.player_up = Player::WHITE;
        gs.moves.push(MoveMade {
            player: Player::BLACK,
            coord: Some(Coord { x: 10, y: 10 }),
            captured: vec![],
            event_id: EventId::new(),
            game_id: GameId(Uuid::new_v4()),
            reply_to: ReqId(Uuid::new_v4()),
        });
        let bytes = gs.serialize().unwrap();
        let back = GameState::from(&bytes).unwrap();
        assert_eq!(back, gs);
    }

    #[test]
    fn player_from_string() {
        assert_eq!(Player::from_str("WHITE"), Player::WHITE);
        assert_eq!(Player::from_str("BLACK"), Player::BLACK);
        assert_eq!(Player::from_str("W"), Player::WHITE);
        assert_eq!(Player::from_str("B"), Player::BLACK);
        assert_eq!(Player::from_str("white"), Player::WHITE);
        assert_eq!(Player::from_str("black"), Player::BLACK);
        assert_eq!(Player::from_str("w"), Player::WHITE);
        assert_eq!(Player::from_str("b"), Player::BLACK);
        assert_eq!(Player::from_str(""), Player::BLACK);
    }
}
