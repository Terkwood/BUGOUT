use crate::repo::{EntryIdRepo, GameLobbyRepo, KeyProvider, RedisRepo};
use crate::stream::{XAdd, XRead};

use std::sync::Arc;

pub struct Components {
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub game_lobby_repo: Box<dyn GameLobbyRepo>,
    pub xread: Box<dyn XRead>,
    pub xadd: Box<dyn XAdd>,
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
            xread: Box::new(arc_client.clone()),
            xadd: Box::new(arc_client),
        }
    }
}
