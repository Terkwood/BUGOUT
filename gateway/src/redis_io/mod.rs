pub mod entry_id_repo;
mod key_provider;
mod namespace;
pub mod stream;
pub mod xadd;
pub mod xread;

pub use key_provider::*;
pub use namespace::*;
pub use xadd::start;

use r2d2_redis::{r2d2, RedisConnectionManager};
use redis_streams::XReadEntryId;

pub type RedisPool = r2d2_redis::r2d2::Pool<r2d2_redis::RedisConnectionManager>;

pub const REDIS_URL: &str = "redis://redis";

pub fn create_pool() -> RedisPool {
    let manager = RedisConnectionManager::new(REDIS_URL).unwrap();
    r2d2::Pool::builder().build(manager).unwrap()
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AllEntryIds {
    pub bot_attached_eid: XReadEntryId,
    pub move_made_eid: XReadEntryId,
}
