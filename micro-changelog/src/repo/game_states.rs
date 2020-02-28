use super::redis_key::*;
use super::{FetchErr, WriteErr};
use micro_model_moves::{GameId, GameState};
use redis_conn_pool::redis::Commands;
use redis_conn_pool::Pool;

const EXPIRY_SECS: usize = 86400;

#[derive(Clone, Debug)]
pub struct GameStatesRepo {
    pub pool: Pool,
    pub hash_key: GameStatesHashKey,
}

impl GameStatesRepo {
    pub fn fetch(&self, game_id: &GameId) -> Result<GameState, FetchErr> {
        let mut conn = self.pool.get().unwrap();
        let key = self.hash_key.value(&game_id);
        let bin_data: Vec<u8> = conn.get(&key)?;
        let r = GameState::from(&bin_data)?;
        // Touch TTL whenever you get the record
        conn.expire(key, EXPIRY_SECS)?;
        Ok(r)
    }

    pub fn write(&self, game_id: GameId, game_state: GameState) -> Result<String, WriteErr> {
        let mut conn = self.pool.get().unwrap();

        let key = self.hash_key.value(&game_id);
        let done = conn.set(&key, game_state.serialize()?)?;
        // Touch TTL whenever you set the record
        conn.expire(key, EXPIRY_SECS)?;
        Ok(done)
    }
}
