use crate::model::*;
use redis::{Client, Commands};
use std::rc::Rc;
pub trait HistoryRepo {
    fn get(&self, game_id: &GameId) -> Result<Option<Vec<Move>>, FetchErr>;
    fn put(&self, game_id: &GameId, moves: Vec<Move>) -> Result<(), WriteErr>;
}

pub struct FetchErr;
pub struct WriteErr;

const EXPIRY_SECS: usize = 86400;
impl HistoryRepo for Rc<Client> {
    fn get(&self, game_id: &GameId) -> Result<Option<Vec<Move>>, FetchErr> {
        if let Ok(mut conn) = self.get_connection() {
            let key = redis_key(game_id);
            let data: Result<Vec<u8>, _> = conn.get(&key).map_err(|_| FetchErr);

            if let Ok(_) = data {
                // Touch TTL whenever you get the record
                conn.expire(&key, EXPIRY_SECS)?;
            }

            data.and_then(|bytes| bincode::deserialize(&bytes).map_err(|_| FetchErr))
        } else {
            Err(FetchErr)
        }
    }

    fn put(&self, game_id: &GameId, moves: Vec<Move>) -> Result<(), WriteErr> {
        let key = redis_key(game_id);
        if let (Ok(mut conn), Ok(bytes)) = (self.get_connection(), bincode::serialize(&moves)) {
            let done = conn.set(&key, bytes).map_err(|_| WriteErr)?;
            conn.expire(&key, EXPIRY_SECS)?;
            Ok(done)
        } else {
            Err(WriteErr)
        }
    }
}

fn redis_key(game_id: &GameId) -> String {
    format!("/BUGOUT/micro_sync/history/{}", game_id.0)
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
