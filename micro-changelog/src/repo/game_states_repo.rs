use super::{FetchErr, WriteErr};
use crate::Components;
use micro_model_moves::{GameId, GameState};
use redis_conn_pool::redis::Commands;

const EXPIRY_SECS: usize = 86400;

pub fn fetch(game_id: &GameId, components: &Components) -> Result<Option<GameState>, FetchErr> {
    let mut conn = components.pool.get().unwrap();
    let key = components.redis_key_provider.game_states(&game_id);
    let bin_data: Option<Vec<u8>> = conn.get(&key)?;
    Ok(match bin_data {
        Some(b) => {
            let r = GameState::from(&b)?;
            // Touch TTL whenever you get the record
            conn.expire(key, EXPIRY_SECS)?;
            Some(r)
        }
        None => None,
    })
}

pub fn write(
    game_id: &GameId,
    game_state: &GameState,
    components: &Components,
) -> Result<String, WriteErr> {
    let mut conn = components.pool.get().unwrap();

    let key = components.redis_key_provider.game_states(game_id);
    let done = conn.set(&key, game_state.serialize()?)?;
    // Touch TTL whenever you set the record
    conn.expire(key, EXPIRY_SECS)?;

    Ok(done)
}
