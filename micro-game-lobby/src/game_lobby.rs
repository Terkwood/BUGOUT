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
            LobbyCommand::Abandon => {
                self.games.remove(&command.game);
            }
            LobbyCommand::Ready => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    #[test]
    fn lobby_execute_bytes() {
        let mut lobby = GameLobby {
            games: HashSet::new(),
        };
        assert!(lobby.as_bytes().is_ok());
        lobby.execute(GameLobbyCommand {
            game: Game {
                game_id: GameId(Uuid::new_v4()),
                board_size: 3,
                creator: SessionId(Uuid::new_v4()),
                visibility: Visibility::Private,
            },
            lobby_command: LobbyCommand::Open,
        });
        assert!(!lobby.games.is_empty());
        assert!(lobby.as_bytes().is_ok());
    }
}
