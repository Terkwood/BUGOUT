use crate::repo::GameLobbyRepo;
use crate::stream::{XAdd, XRead};

use std::rc::Rc;

pub struct Components {
    pub game_lobby_repo: Box<dyn GameLobbyRepo>,
    pub xread: Box<dyn XRead>,
    pub xadd: Box<dyn XAdd>,
}

const REDIS_URL: &str = "redis://redis/";

impl Default for Components {
    fn default() -> Self {
        let client = Rc::new(redis::Client::open(REDIS_URL).expect("redis client"));
        Components {
            game_lobby_repo: Box::new(client.clone()),
            xread: Box::new(client.clone()),
            xadd: Box::new(client),
        }
    }
}
