use super::{FetchErr, WriteErr};
use lobby_model::GameLobby;
use redis::Client;
use redis::Commands;
use std::rc::Rc;

pub trait GameLobbyRepo {
    fn get(&self) -> Result<GameLobby, FetchErr>;
    fn put(&self, game_lobby: GameLobby) -> Result<(), WriteErr>;
}

impl GameLobbyRepo for Rc<Client> {
    fn get(&self) -> Result<GameLobby, FetchErr> {
        if let Ok(mut conn) = self.get_connection() {
            let data: Result<Vec<u8>, _> = conn.get(super::GAME_LOBBY_KEY).map_err(|_| FetchErr);

            data.and_then(|bytes| bincode::deserialize(&bytes).map_err(|_| FetchErr))
        } else {
            Err(FetchErr)
        }
    }
    fn put(&self, game_lobby: GameLobby) -> Result<(), WriteErr> {
        if let (Ok(mut conn), Ok(bytes)) = (self.get_connection(), bincode::serialize(&game_lobby))
        {
            conn.set(super::GAME_LOBBY_KEY, bytes).map_err(|_| WriteErr)
        } else {
            Err(WriteErr)
        }
    }
}
