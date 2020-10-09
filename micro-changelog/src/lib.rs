mod model;
pub mod repo;
pub mod stream;

use log::info;
use repo::redis_key::KeyProvider;
use std::rc::Rc;

pub struct Components {
    pub client: Rc<redis::Client>,
    pub redis_key_provider: KeyProvider,
}

impl Default for Components {
    fn default() -> Self {
        let client = Rc::new(redis::Client::open("redis://redis").expect("client"));
        info!("Connected to redis");
        Components {
            client,
            redis_key_provider: KeyProvider::default(),
        }
    }
}
