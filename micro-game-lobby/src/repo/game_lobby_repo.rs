use crate::game_lobby::GameLobby;

use super::{FetchErr, WriteErr};

trait GameLobbyRepo {
    fn get() -> Result<GameLobby, FetchErr>;
    fn put(game_lobby: GameLobby) -> Result<(), WriteErr>;
}
