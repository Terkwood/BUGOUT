use crate::model::*;
use redis::Client;
use std::rc::Rc;

pub trait PrefsRepo {
    fn get(&self, game_id: &GameId) -> Result<Vec<SessionColorPref>, FetchErr>;
    fn put(&self, game_id: &GameId, scp: SessionColorPref) -> Result<(), WriteErr>;
}

pub struct FetchErr;
pub struct WriteErr;

const EXPIRY_SECS: usize = 86400;

impl PrefsRepo for Rc<Client> {
    fn get(&self, game_id: &GameId) -> Result<Vec<SessionColorPref>, FetchErr> {
        todo!("get redis list")
    }

    fn put(&self, game_id: &GameId, scp: SessionColorPref) -> Result<(), WriteErr> {
        todo!("write redis list")
    }
}

impl From<redis::RedisError> for FetchErr {
    fn from(_: redis::RedisError) -> Self {
        Self
    }
}
impl From<redis::RedisError> for WriteErr {
    fn from(_: redis::RedisError) -> Self {
        Self
    }
}
