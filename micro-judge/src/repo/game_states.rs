use crate::io::conn_pool::Pool;
use crate::io::redis_keys::{game_states_key, RedisKeyNamespace};
use crate::io::FetchErr;

use crate::model::{GameId, GameState};
use bincode;
use r2d2_redis::redis;
use redis::Commands;

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

#[derive(Debug)]
pub enum WriteErr {
    Redis(redis::RedisError),
    Serialization(std::boxed::Box<bincode::ErrorKind>),
}
impl From<std::boxed::Box<bincode::ErrorKind>> for WriteErr {
    fn from(ek: std::boxed::Box<bincode::ErrorKind>) -> Self {
        WriteErr::Serialization(ek)
    }
}
impl From<redis::RedisError> for WriteErr {
    fn from(r: redis::RedisError) -> Self {
        WriteErr::Redis(r)
    }
}

impl GameState {
    pub fn serialize(&self) -> Result<Vec<u8>, std::boxed::Box<bincode::ErrorKind>> {
        Ok(bincode::serialize(&self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_game_state_ser_basic() {
        let gs = GameState::default();
        let result = gs.serialize();
        assert!(result.is_ok());
        assert!(result.unwrap().len() > 0)
    }

    #[test]
    fn there_and_back() {
        let gs = GameState::default();
        let bytes = gs.serialize().unwrap();
        let back = GameState::from(&bytes).unwrap();
        assert_eq!(back, gs);
    }
}
