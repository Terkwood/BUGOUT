use crate::model::*;
use redis::Client;
use std::rc::Rc;
pub trait HistoryRepo {
    fn provide(&self, game_id: GameId) -> Option<Vec<Move>>;
}
impl HistoryRepo for Rc<Client> {
    fn provide(&self, game_id: GameId) -> Option<Vec<Move>> {
        todo!()
    }
}
