use core_model::*;
use lobby_model::*;

pub trait GameLobbyOps {
    fn open(&self, game: Game) -> Self;
    fn ready(&self, game: &Game) -> Self;
    fn abandon(&self, session_id: &SessionId) -> Self;
}

impl GameLobbyOps for GameLobby {
    fn open(&self, game: Game) -> Self {
        let mut r = self.clone();
        r.games.insert(game);
        r
    }

    fn ready(&self, game: &Game) -> Self {
        let mut r = self.clone();
        r.games.remove(&game);
        r
    }

    fn abandon(&self, session_id: &SessionId) -> Self {
        let mut r = self.clone();
        if let Some(game) = r.games.clone().iter().find(|g| &g.creator == session_id) {
            r.games.remove(game);
        }
        r
    }
}

trait AsBytes {
    fn as_bytes(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>>;
}

impl AsBytes for GameLobby {
    fn as_bytes(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
        bincode::serialize(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lobby_execute_bytes() {
        let lobby = GameLobby::default();
        assert!(lobby.as_bytes().is_ok());
        let next = lobby.open(Game {
            game_id: GameId::new(),
            board_size: 3,
            creator: SessionId::new(),
            visibility: Visibility::Private,
        });
        assert!(!next.games.is_empty());
        assert!(next.as_bytes().is_ok());
    }
}
