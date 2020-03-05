use crate::*;
use std::collections::HashSet;
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone)]
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
}
