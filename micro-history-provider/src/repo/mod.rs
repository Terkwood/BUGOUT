use crate::model::*;
use redis::Client;
use std::rc::Rc;
pub trait HistoryRepo {
    fn get(&self, game_id: &GameId) -> Result<Option<Vec<Move>>, FetchErr>;
    fn put(&self, game_id: &GameId, moves: Vec<Move>) -> Result<(), WriteErr>;
}
impl HistoryRepo for Rc<Client> {
    fn get(&self, game_id: &GameId) -> Result<Option<Vec<Move>>, FetchErr> {
        todo!()
    }

    fn put(&self, game_id: &GameId, moves: Vec<Move>) -> Result<(), WriteErr> {
        todo!()
    }
}
pub struct FetchErr;
pub struct WriteErr;
