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
            let first_key = redis_key(&game_ready.sessions.0);
            let second_key = redis_key(&game_ready.sessions.0);
            if let Ok(bytes) = bincode::serialize(&game_ready) {
                let first_done: Result<(), _> = conn.set(&first_key, bytes.clone());
                if let Ok(_) = first_done {
                    touch_ttl(&mut conn, &first_key)
                }
                let second_done: Result<(), _> = conn.set(&second_key, bytes);
                if let Ok(_) = second_done {
                    touch_ttl(&mut conn, &second_key)
                }

                first_done.and_then(|_| second_done).map_err(|_| WriteErr)
            } else {
                Err(WriteErr)
            }
        } else {
            Err(WriteErr)
        }
    }
}

fn redis_key(session_id: &SessionId) -> String {
    format!("/BUGOUT/micro_color_chooser/game_ready/{}", session_id.0)
}
