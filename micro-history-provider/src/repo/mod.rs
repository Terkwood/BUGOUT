use crate::model::*;
use redis::{Client, Commands};
use std::rc::Rc;
pub trait HistoryRepo {
    fn get(&self, game_id: &GameId) -> Result<Option<Vec<Move>>, FetchErr>;
    fn put(&self, game_id: &GameId, moves: Vec<Move>) -> Result<(), WriteErr>;
}

pub struct FetchErr;
pub struct WriteErr;

impl HistoryRepo for Rc<Client> {
    fn get(&self, game_id: &GameId) -> Result<Option<Vec<Move>>, FetchErr> {
        if let Ok(mut conn) = self.get_connection() {
            let data: Result<Vec<u8>, _> = conn.get(redis_key(game_id)).map_err(|_| FetchErr);

            data.and_then(|bytes| bincode::deserialize(&bytes).map_err(|_| FetchErr))
        } else {
            Err(FetchErr)
        }
    }

    fn put(&self, game_id: &GameId, moves: Vec<Move>) -> Result<(), WriteErr> {
        todo!();
        todo!("ttl")
    }
}

fn redis_key(game_id: &GameId) -> String {
    format!("/BUGOUT/micro_history_provider/history/{}", game_id.0)
}
