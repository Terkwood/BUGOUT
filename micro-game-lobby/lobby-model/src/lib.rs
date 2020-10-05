extern crate serde_derive;

pub mod api;

use core_model::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Game {
    pub game_id: GameId,
    pub visibility: Visibility,
    pub creator: SessionId,
    pub board_size: u16,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
}

/// A structure representing all games which are
/// waiting for a second player to join
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLobby {
    pub games: HashSet<Game>,
}

impl Default for GameLobby {
    fn default() -> Self {
        GameLobby {
            games: HashSet::new(),
        }
    }
}
