use super::*;

/// Associates SessionIds with GameIds and allows retrieval by SessionId
pub trait SessionGameRepo {
    fn get(&self, session_id: &SessionId) -> Result<Option<GameId>, FetchErr>;
    fn put(&self, session_id: &SessionId, game_id: &GameId) -> Result<(), WriteErr>;
}

impl SessionGameRepo for Rc<Client> {
    fn get(&self, session_id: &SessionId) -> Result<Option<GameId>, FetchErr> {
        todo!("redis game repo get")
    }

    fn put(&self, session_id: &SessionId, game_id: &GameId) -> Result<(), WriteErr> {
        todo!("redis game repo put")
    }
}
