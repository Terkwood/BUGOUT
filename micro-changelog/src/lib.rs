extern crate bincode;
pub extern crate micro_model_moves;
extern crate redis_streams;
// extern crate serde;
// extern crate serde_derive;
mod model;
pub mod repo;
pub mod stream;

pub use redis_conn_pool;
pub use redis_conn_pool::{r2d2, r2d2_redis, redis, RedisHostUrl};
use repo::redis_key::HashKeyProvider;

pub struct Components {
    pub pool: redis_conn_pool::Pool,
    pub hash_key_provider: HashKeyProvider,
}

impl Default for Components {
    fn default() -> Self {
        let pool = redis_conn_pool::create(RedisHostUrl::default());
        println!("Connected to redis");
        Components {
            pool,
            hash_key_provider: HashKeyProvider::default(),
        }
    }
}
