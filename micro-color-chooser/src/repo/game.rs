use super::*;

/// Associates SessionIds with GameIds and allows retrieval by SessionId
pub trait SessionGameRepo {
    fn get(&self, session_id: &SessionId) -> Result<Option<SessionGame>, FetchErr>;
    fn put(&self, session_game: SessionGame) -> Result<(), WriteErr>;
}

impl SessionGameRepo for Rc<Client> {
    fn get(&self, session_id: &SessionId) -> Result<Option<SessionGame>, FetchErr> {
        todo!("redis game repo get")
    }

    fn put(&self, session_game: SessionGame) -> Result<(), WriteErr> {
        todo!("redis game repo put")
    }
}
