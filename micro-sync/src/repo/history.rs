use super::*;
use core_model::*;
use redis::{Client, Commands};
use std::rc::Rc;
use sync_model::Move;

pub trait HistoryRepo {
    fn get(&self, game_id: &GameId) -> Result<Option<Vec<Move>>, FetchErr>;
    fn put(&self, game_id: &GameId, moves: Vec<Move>) -> Result<(), WriteErr>;
}

impl HistoryRepo for Rc<Client> {
    fn get(&self, game_id: &GameId) -> Result<Option<Vec<Move>>, FetchErr> {
        if let Ok(mut conn) = self.get_connection() {
            let key = redis_key(game_id);
            let data: Result<Vec<u8>, _> = conn.get(&key).map_err(|_| FetchErr);

            if data.is_ok() {
                touch_ttl(&mut conn, &key)
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
            touch_ttl(&mut conn, &key);
            Ok(done)
        } else {
            Err(WriteErr)
        }
    }
}

fn redis_key(game_id: &GameId) -> String {
    format!("/BUGOUT/micro_sync/history/{}", game_id.0)
}
