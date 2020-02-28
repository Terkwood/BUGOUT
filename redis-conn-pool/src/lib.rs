pub extern crate r2d2_redis;
pub extern crate redis;

pub use r2d2_redis::{r2d2, RedisConnectionManager};

pub const DEFAULT_HOST_URL: &str = "redis://redis";

pub type Pool = r2d2_redis::r2d2::Pool<r2d2_redis::RedisConnectionManager>;

pub struct RedisHostUrl(pub String);
impl Default for RedisHostUrl {
    fn default() -> Self {
        RedisHostUrl(DEFAULT_HOST_URL.to_string())
    }
}

pub fn create(host_url: RedisHostUrl) -> Pool {
    let manager = RedisConnectionManager::new(host_url.0).unwrap();
    r2d2::Pool::builder().build(manager).unwrap()
}
