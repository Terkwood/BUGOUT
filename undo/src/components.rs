use crate::stream::{XAck, XAdd};

use std::rc::Rc;

pub struct Components {
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
            xadd: Box::new(client.clone()),
            xack: Box::new(client.clone()),
        }
    }
}
