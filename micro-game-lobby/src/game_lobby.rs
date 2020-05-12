use crate::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashSet;

/// A structure representing all games which are
/// waiting for a second player to join
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLobby {
    pub games: HashSet<Game>,
}

impl GameLobby {
    pub fn open(&self, game: Game) -> Self {
        let mut r = self.clone();
        r.games.insert(game);
        r
    }

    pub fn ready(&self, game: &Game) -> Self {
        let mut r = self.clone();
        r.games.remove(&game);
        r
    }

    pub fn abandon(&self, game: &Game) -> Self {
        let mut r = self.clone();
        r.games.remove(game);
        r
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
        bincode::serialize(self)
    }
}
impl Default for GameLobby {
    fn default() -> Self {
        GameLobby {
            games: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Game {
    pub game_id: GameId,
    pub visibility: Visibility,
    pub creator: SessionId,
    pub board_size: u16,
}
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    #[test]
    fn lobby_execute_bytes() {
        let lobby = GameLobby {
            games: HashSet::new(),
        };
        assert!(lobby.as_bytes().is_ok());
        let next = lobby.open(Game {
            game_id: GameId(Uuid::new_v4()),
            board_size: 3,
            creator: SessionId(Uuid::new_v4()),
            visibility: Visibility::Private,
        });
        assert!(!next.games.is_empty());
        assert!(next.as_bytes().is_ok());
    }
}
