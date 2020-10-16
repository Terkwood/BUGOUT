use super::*;
use color_model::api::GameReady;
use core_model::SessionId;
use log::trace;
use redis::Commands;

/// Associates SessionIds with GameIds and allows retrieval by SessionId
pub trait GameReadyRepo {
    /// Get a game ready record for this session, if it exists
    /// And then update the record's TTL
    fn get(&self, session_id: &SessionId) -> Result<Option<GameReady>, FetchErr>;
    /// Save a game ready record, associating it with both session IDs
    /// found in its `sessions` field.  Updates record TTL.
    fn put(&self, game_ready: GameReady) -> Result<(), WriteErr>;
}

impl GameReadyRepo for Rc<Client> {
    fn get(&self, session_id: &SessionId) -> Result<Option<GameReady>, FetchErr> {
        if let Ok(mut conn) = self.get_connection() {
            let key = redis_key(session_id);
            let data: Result<Option<Vec<u8>>, _> = conn.get(&key).map_err(|_| FetchErr::Fetch);

            match data {
                Ok(Some(bytes)) => {
                    touch_ttl(&mut conn, &key);
                    trace!("Fetch game ready for {:?}", &session_id);
                    bincode::deserialize(&bytes).map_err(|_| FetchErr::Deser)
                }
                Ok(None) => Ok(None),
                Err(_) => Err(FetchErr::Fetch),
            }
        } else {
            Err(FetchErr::Conn)
        }
    }

    fn put(&self, game_ready: GameReady) -> Result<(), WriteErr> {
        let c = self.get_connection();
        let s = bincode::serialize(&game_ready);
        if let (Ok(mut conn), Ok(bytes)) = (c, s) {
            let first_key = redis_key(&game_ready.sessions.0);
            let second_key = redis_key(&game_ready.sessions.1);

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
    }
}

fn redis_key(session_id: &SessionId) -> String {
    format!("/BUGOUT/micro_color_chooser/game_ready/{}", session_id.0)
}
