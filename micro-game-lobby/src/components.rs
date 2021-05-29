use crate::repo::GameLobbyRepo;
use crate::stream::XAdd;
use redis_streams::{RedisSortedStreams, SortedStreams};
use std::rc::Rc;

pub struct Components {
    pub game_lobby_repo: Box<dyn GameLobbyRepo>,
    pub xadd: Box<dyn XAdd>,
    pub sorted_streams: Box<dyn SortedStreams>,
}

const REDIS_URL: &str = "redis://redis/";

pub fn redis_client() -> Rc<redis::Client> {
    Rc::new(redis::Client::open(REDIS_URL).expect("redis client"))
}

impl Components {
    pub fn new(client: Rc<redis::Client>) -> Self {
        Components {
            game_lobby_repo: Box::new(client.clone()),
            xadd: Box::new(client),
            sorted_streams: todo!(),
        }
    }
}
