use super::{FetchErr, WriteErr};
use crate::Components;
use core_model::GameId;
use move_model::GameState;
use redis::Commands;

const EXPIRY_SECS: usize = 86400;

pub fn fetch(game_id: &GameId, components: &Components) -> Result<Option<GameState>, FetchErr> {
    let mut conn = components.client.get_connection().expect("fetch conn");
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
    let mut conn = components.client.get_connection().expect("write conn");

    let key = components.redis_key_provider.game_states(game_id);
    let done = conn.set(&key, game_state.serialize()?)?;
    // Touch TTL whenever you set the record
    conn.expire(key, EXPIRY_SECS)?;

    Ok(done)
}
