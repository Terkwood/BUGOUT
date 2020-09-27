use super::*;
use crate::api::GameReady;
use redis::Commands;

/// Associates SessionIds with GameIds and allows retrieval by SessionId
pub trait GameReadyRepo {
    fn get(&self, session_id: &SessionId) -> Result<Option<GameReady>, FetchErr>;
    fn put(&self, game_ready: GameReady) -> Result<(), WriteErr>;
}

impl GameReadyRepo for Rc<Client> {
    /// Get a game ready record for this session, if it exists
    /// And then update the record's TTL
    fn get(&self, session_id: &SessionId) -> Result<Option<GameReady>, FetchErr> {
        if let Ok(mut conn) = self.get_connection() {
            let key = redis_key(session_id);
            let data: Result<Vec<u8>, _> = conn.get(&key).map_err(|_| FetchErr);

            if let Ok(_) = data {
                touch_ttl(&mut conn, &key)
            }

            data.and_then(|bytes| bincode::deserialize(&bytes).map_err(|_| FetchErr))
        } else {
            Err(FetchErr)
        }
    }

    fn put(&self, game_ready: GameReady) -> Result<(), WriteErr> {
        if let Ok(mut conn) = self.get_connection() {
            todo!("redis game repo put first record");
            todo!("redis game repo put second record");
            todo!("touch ttl")
        } else {
            Err(WriteErr)
        }
    }
}

fn redis_key(session_id: &SessionId) -> String {
    format!("/BUGOUT/micro_color_chooser/game_ready/{}", session_id.0)
}
