use super::RepoErr;
use core_model::GameId;
use move_model::GameState;
use redis::Client;
use redis::Commands;
use std::rc::Rc;

pub trait GameStateRepo {
    fn get(&self, game_id: &GameId) -> Result<Option<GameState>, RepoErr>;
    fn put(&self, game_state: &GameState) -> Result<(), RepoErr>;
}

impl GameStateRepo for Rc<Client> {
    fn get(&self, game_id: &GameId) -> Result<Option<GameState>, RepoErr> {
        let mut conn = self.get_connection()?;
        let data: Option<Vec<u8>> = conn.get(key(game_id))?;
        Ok(if let Some(bytes) = data {
            Some(bincode::deserialize(&bytes)?)
        } else {
            None
        })
    }

    fn put(&self, game_state: &GameState) -> Result<(), RepoErr> {
        log::info!(
            "🐌 game turn: {}, player up: {:?}, moves: {}",
            game_state.turn,
            game_state.player_up,
            game_state.moves.len()
        );
        let mut conn = self.get_connection()?;
        let bytes = bincode::serialize(&game_state)?;
        conn.set(key(&game_state.game_id), bytes)?;
        Ok(())
    }
}

fn key(game_id: &GameId) -> String {
    format!("/BUGOUT/undo/game_state/{}", game_id.0.to_string())
}
