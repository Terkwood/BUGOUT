use crate::*;
use bincode;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GameLobby {
    games: HashSet<Game>,
}

impl GameLobby {
    pub fn execute(&mut self, command: GameLobbyCommand) {
        match command.lobby_command {
            LobbyCommand::Open => {
                self.games.insert(command.game);
            }
            _ => {
                self.games.remove(&command.game);
            }
        }
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
        bincode::serialize(self)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Game {
    pub game_id: GameId,
    pub visibility: Visibility,
    pub creator: SessionId,
    pub board_size: u16,
}

#[derive(Debug, Clone)]
pub struct GameLobbyCommand {
    pub game: Game,
    pub lobby_command: LobbyCommand,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LobbyCommand {
    Open,
    Ready,
    Abandon,
}
