use crate::game_lobby::GameLobby;

use super::{FetchErr, WriteErr};

pub trait GameLobbyRepo {
    fn get(&self) -> Result<GameLobby, FetchErr>;
    fn put(&self, game_lobby: GameLobby) -> Result<(), WriteErr>;
}

impl GameLobbyRepo for super::RedisRepo {
    fn get(&self) -> Result<GameLobby, FetchErr> {
        todo!()
    }
    fn put(&self, _game_lobby: GameLobby) -> Result<(), WriteErr> {
        todo!()
    }
}
