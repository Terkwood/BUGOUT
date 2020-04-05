pub mod entry_id_repo;
mod key_provider;
mod namespace;
pub mod stream;
pub mod xadd;

pub use key_provider::*;
pub use namespace::*;
pub use xadd::xadd_commands;

use r2d2_redis::{r2d2, RedisConnectionManager};

pub type RedisPool = r2d2_redis::r2d2::Pool<r2d2_redis::RedisConnectionManager>;

pub const REDIS_URL: &str = "redis://redis";

pub fn create_pool() -> RedisPool {
    let manager = RedisConnectionManager::new(REDIS_URL).unwrap();
    r2d2::Pool::builder().build(manager).unwrap()
}
