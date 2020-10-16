use super::{FetchErr, WriteErr};
use lobby_model::GameLobby;
use redis::Client;
use redis::Commands;
use std::rc::Rc;

pub trait GameLobbyRepo {
    fn get(&self) -> Result<GameLobby, FetchErr>;
    fn put(&self, game_lobby: &GameLobby) -> Result<(), WriteErr>;
}

impl GameLobbyRepo for Rc<Client> {
    fn get(&self) -> Result<GameLobby, FetchErr> {
        if let Ok(mut conn) = self.get_connection() {
            let data: Result<Option<Vec<u8>>, _> = conn
                .get(super::GAME_LOBBY_KEY)
                .map_err(|_| FetchErr::RedisCall);
            match data {
                Ok(Some(bytes)) => Ok(bincode::deserialize(&bytes)?),
                Ok(None) => Ok(GameLobby::default()),
                Err(_) => Err(FetchErr::RedisCall),
            }
        } else {
            Err(FetchErr::Conn)
        }
    }
    fn put(&self, game_lobby: &GameLobby) -> Result<(), WriteErr> {
        if let (Ok(mut conn), Ok(bytes)) = (self.get_connection(), bincode::serialize(&game_lobby))
        {
            conn.set(super::GAME_LOBBY_KEY, bytes).map_err(|_| WriteErr)
        } else {
            Err(WriteErr)
        }
    }
}
