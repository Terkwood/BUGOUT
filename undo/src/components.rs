use crate::repo::{BotRepo, GameStateRepo};
use crate::stream::{XAck, XAdd, XRead};

use std::rc::Rc;

pub struct Components {
    pub xadd: Box<dyn XAdd>,
    pub xack: Box<dyn XAck>,
    pub xread: Box<dyn XRead>,
    pub bot_repo: Box<dyn BotRepo>,
    pub game_state_repo: Box<dyn GameStateRepo>,
}

const REDIS_URL: &str = "redis://redis/";

pub fn redis_client() -> Rc<redis::Client> {
    Rc::new(redis::Client::open(REDIS_URL).expect("redis client"))
}

impl Components {
    pub fn new(client: Rc<redis::Client>) -> Self {
        Components {
            bot_repo: Box::new(client.clone()),
            game_state_repo: Box::new(client.clone()),
            xadd: Box::new(client.clone()),
            xack: Box::new(client.clone()),
            xread: Box::new(client),
        }
    }
}
