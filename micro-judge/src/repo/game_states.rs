use crate::io::redis_keys::{game_states_key, RedisKeyNamespace};
use crate::io::{FetchErr, WriteErr};

use core_model::GameId;
use move_model::GameState;
use redis::{Client, Commands};

const EXPIRY_SECS: usize = 86400;

#[derive(Clone, Debug)]
pub struct GameStatesRepo {
    pub namespace: RedisKeyNamespace,
    pub client: std::rc::Rc<Client>,
}

impl GameStatesRepo {
    pub fn fetch(&self, game_id: &GameId) -> Result<Option<GameState>, FetchErr> {
        let mut conn = self.client.get_connection().unwrap();
        let key = game_states_key(&self.namespace, &game_id);
        let bin_data: Option<Vec<u8>> = conn.get(&key)?;
        let r = if let Some(b) = bin_data {
            // Touch TTL whenever you get the record
            conn.expire(key, EXPIRY_SECS)?;
            Some(GameState::from(&b)?)
        } else {
            None
        };

        Ok(r)
    }

    pub fn write(&self, game_id: &GameId, game_state: &GameState) -> Result<String, WriteErr> {
        let mut conn = self.client.get_connection().unwrap();

        let key = game_states_key(&self.namespace, &game_id);
        let done = conn.set(&key, game_state.serialize()?)?;
        // Touch TTL whenever you set the record
        conn.expire(key, EXPIRY_SECS)?;
        Ok(done)
    }
}
