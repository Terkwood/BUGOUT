mod key_provider;
mod namespace;
pub mod stream;
mod unacknowledged;
mod xack;
pub mod xadd;
pub mod xread;

pub use key_provider::*;
pub use namespace::*;
use unacknowledged::*;
pub use xadd::start;

use redis::Client;
use redis_streams::XReadEntryId;
use std::sync::Arc;

pub const GROUP_NAME: &str = "micro-gateway";

pub const REDIS_URL: &str = "redis://redis";

pub fn create_redis_client() -> Arc<Client> {
    Arc::new(Client::open(REDIS_URL).expect("redis client"))
}
