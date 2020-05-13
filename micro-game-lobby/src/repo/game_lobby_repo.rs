use super::{FetchErr, WriteErr};
use crate::game_lobby::GameLobby;

use redis::Client;
use std::rc::Rc;

pub trait GameLobbyRepo {
    fn get(&self) -> Result<GameLobby, FetchErr>;
    fn put(&self, game_lobby: GameLobby) -> Result<(), WriteErr>;
}

impl GameLobbyRepo for Rc<Client> {
    fn get(&self) -> Result<GameLobby, FetchErr> {
        todo!(" KEY IS CONST STRING IN mod.rs")
    }
    fn put(&self, _game_lobby: GameLobby) -> Result<(), WriteErr> {
        todo!()
    }
}
