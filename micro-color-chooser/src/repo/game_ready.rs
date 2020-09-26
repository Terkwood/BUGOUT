use super::*;
use crate::api::GameReady;

/// Associates SessionIds with GameIds and allows retrieval by SessionId
pub trait GameReadyRepo {
    fn get(&self, session_id: &SessionId) -> Result<Option<GameReady>, FetchErr>;
    fn put(&self, game_ready: GameReady) -> Result<(), WriteErr>;
}

impl GameReadyRepo for Rc<Client> {
    fn get(&self, session_id: &SessionId) -> Result<Option<GameReady>, FetchErr> {
        todo!("redis game repo get")
    }

    fn put(&self, game_ready: GameReady) -> Result<(), WriteErr> {
        todo!("redis game repo put")
    }
}
