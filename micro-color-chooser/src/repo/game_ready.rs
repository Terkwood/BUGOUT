use super::*;
use crate::api::GameReady;
use redis::Commands;

/// Associates SessionIds with GameIds and allows retrieval by SessionId
pub trait GameReadyRepo {
    fn get(&self, session_id: &SessionId) -> Result<Option<GameReady>, FetchErr>;
    fn put(&self, game_ready: GameReady) -> Result<(), WriteErr>;
}

impl GameReadyRepo for Rc<Client> {
    fn get(&self, session_id: &SessionId) -> Result<Option<GameReady>, FetchErr> {
        if let Ok(mut conn) = self.get_connection() {
            let key = redis_key(session_id);
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

    fn put(&self, game_ready: GameReady) -> Result<(), WriteErr> {
        if let Ok(mut conn) = self.get_connection() {
            todo!("redis game repo put")
        } else {
            Err(WriteErr)
        }
    }
}

fn redis_key(session_id: &SessionId) -> String {
    todo!()
}
