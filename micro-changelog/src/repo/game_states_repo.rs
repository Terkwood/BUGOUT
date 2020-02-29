use super::redis_key::*;
use super::{FetchErr, WriteErr};
use crate::Components;
use micro_model_moves::{GameId, GameState};
use redis_conn_pool::redis::Commands;
use redis_conn_pool::{Pool, RedisHostUrl};
const EXPIRY_SECS: usize = 86400;

pub fn fetch(game_id: &GameId, components: &Components) -> Result<GameState, FetchErr> {
    let mut conn = components.pool.get().unwrap();
    let key = components.hash_key_provider.game_states(&game_id);
    let bin_data: Vec<u8> = conn.get(&key)?;
    let r = GameState::from(&bin_data)?;
    // Touch TTL whenever you get the record
    conn.expire(key, EXPIRY_SECS)?;
    Ok(r)
}

pub fn write(
    game_id: GameId,
    game_state: GameState,
    components: &Components,
) -> Result<String, WriteErr> {
    let mut conn = components.pool.get().unwrap();

    let key = components.hash_key_provider.game_states(&game_id);
    let done = conn.set(&key, game_state.serialize()?)?;
    // Touch TTL whenever you set the record
    conn.expire(key, EXPIRY_SECS)?;
    Ok(done)
}
