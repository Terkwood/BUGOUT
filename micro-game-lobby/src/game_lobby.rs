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
    fn lobby_as_bytes() {
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

    #[test]
    fn lobby_open() {
        let lobby = GameLobby::default();

        let one = lobby.open(Game {
            game_id: GameId::new(),
            board_size: 19,
            creator: SessionId::new(),
            visibility: Visibility::Public,
        });
        assert_eq!(one.games.len(), 1);
        let two = one.open(Game {
            game_id: GameId::new(),
            board_size: 13,
            creator: SessionId::new(),
            visibility: Visibility::Private,
        });
        assert_eq!(two.games.len(), 2)
    }

    #[test]
    fn lobby_ready() {
        let lobby = GameLobby::default();

        let game = Game {
            game_id: GameId::new(),
            board_size: 19,
            creator: SessionId::new(),
            visibility: Visibility::Public,
        };

        let one = lobby.open(game.clone());
        assert_eq!(one.games.len(), 1);
        let done = one.ready(&game);
        assert!(done.games.is_empty())
    }

    #[test]
    fn lobby_abandon() {
        let lobby = GameLobby::default();

        let sid = SessionId::new();
        let creator = sid.clone();
        let one = lobby.open(Game {
            game_id: GameId::new(),
            board_size: 19,
            creator,
            visibility: Visibility::Public,
        });
        assert_eq!(one.games.len(), 1);
        let done = one.abandon(&sid);
        assert!(done.games.is_empty());
    }
}
