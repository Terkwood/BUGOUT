use super::conn_pool::Pool;
use super::redis_keys::{game_states_key, Namespace};
use super::FetchErr;

use crate::model::{GameId, GameState};
use bincode;
use r2d2_redis::redis;
use redis::Commands;

pub fn fetch(game_id: &GameId, ns: &Namespace, pool: &Pool) -> Result<GameState, FetchErr> {
    let mut conn = pool.get().unwrap();
    let key = game_states_key(ns, &game_id);
    let bin_data: Vec<u8> = conn.get(key)?;
    Ok(GameState::from(&bin_data)?)
}

pub fn write(
    game_id: GameId,
    game_state: GameState,
    ns: &Namespace,
    pool: &Pool,
) -> Result<String, WriteErr> {
    let mut conn = pool.get().unwrap();

    Ok(conn.set(game_states_key(ns, &game_id), game_state.serialize()?)?)
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
