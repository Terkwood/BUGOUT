mod game_lobby_repo;
pub use game_lobby_repo::*;

#[derive(Debug)]
pub enum FetchErr {
    Deser,
    RedisCall,
    Conn,
}

#[derive(Debug)]
pub struct WriteErr;

pub const GAME_LOBBY_KEY: &str = "/BUGOUT/micro_game_lobby/game_lobby";

impl From<Box<bincode::ErrorKind>> for FetchErr {
    fn from(_: Box<bincode::ErrorKind>) -> Self {
        FetchErr::Deser
    }
}
