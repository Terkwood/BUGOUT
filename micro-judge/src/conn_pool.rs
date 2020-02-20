use r2d2_redis::{r2d2, RedisConnectionManager};
const HOST_URL: &str = "redis://redis";

pub type Pool = r2d2_redis::r2d2::Pool<r2d2_redis::RedisConnectionManager>;

pub fn create() -> Pool {
    let manager = RedisConnectionManager::new(HOST_URL).unwrap();
    r2d2::Pool::builder().build(manager).unwrap()
}
