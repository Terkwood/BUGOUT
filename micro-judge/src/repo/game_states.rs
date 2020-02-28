use crate::io::conn_pool::Pool;
use crate::io::redis_keys::{game_states_key, RedisKeyNamespace};
use crate::io::{FetchErr, WriteErr};

use crate::io::redis::Commands;
use crate::model::{GameId, GameState};

const EXPIRY_SECS: usize = 86400;

#[derive(Clone, Debug)]
pub struct GameStatesRepo {
    pub namespace: RedisKeyNamespace,
    pub pool: Pool,
}

impl GameStatesRepo {
    pub fn fetch(&self, game_id: &GameId) -> Result<GameState, FetchErr> {
        let mut conn = self.pool.get().unwrap();
        let key = game_states_key(&self.namespace, &game_id);
        let bin_data: Vec<u8> = conn.get(&key)?;
        let r = GameState::from(&bin_data)?;
        // Touch TTL whenever you get the record
        conn.expire(key, EXPIRY_SECS)?;
        Ok(r)
    }

    pub fn write(&self, game_id: GameId, game_state: GameState) -> Result<String, WriteErr> {
        let mut conn = self.pool.get().unwrap();

        let key = game_states_key(&self.namespace, &game_id);
        let done = conn.set(&key, game_state.serialize()?)?;
        // Touch TTL whenever you set the record
        conn.expire(key, EXPIRY_SECS)?;
        Ok(done)
    }
}
