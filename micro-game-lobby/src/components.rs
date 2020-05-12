use crate::repo::{EntryIdRepo, GameLobbyRepo, KeyProvider, RedisRepo};
use crate::stream::{RedisXRead, XRead};

use std::sync::Arc;

pub struct Components {
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub game_lobby_repo: Box<dyn GameLobbyRepo>,
    pub xreader: Box<dyn XRead>,
}

impl Default for Components {
    fn default() -> Self {
        let client = redis::Client::open("redis://redis/").unwrap();
        let arc_client = Arc::new(client);
        Components {
            entry_id_repo: Box::new(RedisRepo {
                client: arc_client.clone(),
                key_provider: KeyProvider::default(),
            }),
            game_lobby_repo: Box::new(RedisRepo {
                client: arc_client.clone(),
                key_provider: KeyProvider::default(),
            }),
            xreader: Box::new(RedisXRead { client: arc_client }),
        }
    }
}
