use crate::repo::HistoryRepo;
use crate::stream::{XAdd, XRead};
use log::info;
use redis::Client;
use std::rc::Rc;

pub struct Components {
    pub history_repo: Box<dyn HistoryRepo>,
    pub xread: Box<dyn XRead>,
    pub xadd: Box<dyn XAdd>,
    pub client: Rc<Client>,
}

const REDIS_URL: &str = "redis://redis/";

impl Default for Components {
    fn default() -> Self {
        let client = Rc::new(Client::open(REDIS_URL).expect("redis client"));
        info!("Connected to redis");
        Components {
            history_repo: Box::new(client.clone()),
            xread: Box::new(client.clone()),
            xadd: Box::new(client.clone()),
            client,
        }
    }
}
