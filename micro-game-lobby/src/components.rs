use crate::repo::{EntryIdRepo, GameLobbyRepo, RedisRepo};
use crate::stream::{RedisXReader, XReader};

use redis_conn_pool::RedisHostUrl;
use std::sync::Arc;

pub struct Components {
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub game_lobby_repo: Box<dyn GameLobbyRepo>,
    pub xreader: Box<dyn XReader>,
}

impl Default for Components {
    fn default() -> Self {
        let pool = redis_conn_pool::create(RedisHostUrl::default());
        let arc_pool = Arc::new(pool);
        Components {
            entry_id_repo: Box::new(RedisRepo {
                pool: arc_pool.clone(),
                key_provider: (),
            }),
            game_lobby_repo: Box::new(RedisRepo {
                pool: arc_pool.clone(),
                key_provider: (),
            }),
            xreader: Box::new(RedisXReader { pool: arc_pool }),
        }
    }
}
