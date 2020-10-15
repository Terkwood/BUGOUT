use crate::repo::GameLobbyRepo;
use crate::stream::{XAck, XAdd, XRead};

use std::rc::Rc;

pub struct Components {
    pub game_lobby_repo: Box<dyn GameLobbyRepo>,
    pub xread: Box<dyn XRead>,
    pub xadd: Box<dyn XAdd>,
    pub xack: Box<dyn XAck>,
}

const REDIS_URL: &str = "redis://redis/";

pub fn redis_client() -> Rc<redis::Client> {
    Rc::new(redis::Client::open(REDIS_URL).expect("redis client"))
}

impl Components {
    pub fn new(client: Rc<redis::Client>) -> Self {
        Components {
            game_lobby_repo: Box::new(client.clone()),
            xread: Box::new(client.clone()),
            xadd: Box::new(client.clone()),
            xack: Box::new(client),
        }
    }
}
