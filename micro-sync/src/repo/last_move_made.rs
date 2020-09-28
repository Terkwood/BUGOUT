use super::*;
use crate::api::MoveMade;
use crate::model::*;
use redis::Client;
use std::rc::Rc;

pub trait LastMoveMadeRepo {
    fn get(&self, game_id: &GameId) -> Result<Option<MoveMade>, FetchErr>;
    fn put(&self, move_made: MoveMade) -> Result<(), WriteErr>;
}

impl LastMoveMadeRepo for Rc<Client> {
    fn get(&self, game_id: &GameId) -> Result<Option<MoveMade>, FetchErr> {
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

    fn put(&self, move_made: MoveMade) -> Result<(), WriteErr> {
        let key = redis_key(&move_made.game_id);
        if let (Ok(mut conn), Ok(bytes)) = (self.get_connection(), bincode::serialize(&move_made)) {
            let done = conn.set(&key, bytes).map_err(|_| WriteErr)?;
            touch_ttl(&mut conn, &key);
            Ok(done)
        } else {
            Err(WriteErr)
        }
    }
}

fn redis_key(game_id: &GameId) -> String {
    format!("/BUGOUT/micro_sync/last_move_made/{}", game_id.0)
}
