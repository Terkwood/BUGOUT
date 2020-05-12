use crate::repo::{EntryIdRepo, GameLobbyRepo, KeyProvider, RedisRepo};
use crate::stream::{XAdd, XRead};

use std::rc::Rc;

pub struct Components {
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub game_lobby_repo: Box<dyn GameLobbyRepo>,
    pub xread: Box<dyn XRead>,
    pub xadd: Box<dyn XAdd>,
}

impl Default for Components {
    fn default() -> Self {
        let client = redis::Client::open("redis://redis/").unwrap();
        let rc_client = Rc::new(client);
        Components {
            entry_id_repo: Box::new(RedisRepo {
                client: rc_client.clone(),
                key_provider: KeyProvider::default(),
            }),
            game_lobby_repo: Box::new(RedisRepo {
                client: rc_client.clone(),
                key_provider: KeyProvider::default(),
            }),
            xread: Box::new(rc_client.clone()),
            xadd: Box::new(rc_client),
        }
    }
}
