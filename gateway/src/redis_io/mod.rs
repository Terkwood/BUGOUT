mod key_provider;
mod namespace;
pub mod stream;

pub use key_provider::*;
pub use namespace::*;

use redis::Client;
use std::sync::Arc;

pub const REDIS_URL: &str = "redis://redis";

pub fn create_redis_client() -> Arc<Client> {
    Arc::new(Client::open(REDIS_URL).expect("redis client"))
}
