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
            todo!("redis game repo get")
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
