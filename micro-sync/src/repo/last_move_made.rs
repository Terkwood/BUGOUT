use super::*;
use crate::api::MoveMade;
use crate::model::*;

pub trait LastMoveMadeRepo {
    fn get(&self, game_id: &GameId) -> Result<Option<MoveMade>, FetchErr>;
    fn put(&self, move_made: MoveMade) -> Result<(), WriteErr>;
}
