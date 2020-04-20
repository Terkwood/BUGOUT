extern crate bincode;
extern crate env_logger;
extern crate log;
pub extern crate micro_model_moves;
extern crate redis_streams;

mod model;
pub mod repo;
pub mod stream;

pub use redis_conn_pool;
pub use redis_conn_pool::{r2d2, r2d2_redis, redis, RedisHostUrl};
use repo::redis_key::KeyProvider;

use log::info;

pub struct Components {
    pub pool: redis_conn_pool::Pool,
    pub redis_key_provider: KeyProvider,
}

impl Default for Components {
    fn default() -> Self {
        let pool = redis_conn_pool::create(RedisHostUrl::default());
        info!("Connected to redis");
        Components {
            pool,
            redis_key_provider: KeyProvider::default(),
        }
    }
}
