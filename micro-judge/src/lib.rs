extern crate r2d2_redis;
extern crate redis;
extern crate uuid;

pub mod conn_pool;
pub mod stream;
mod topics;

use uuid::Uuid;

#[derive(Debug)]
pub struct GameId(Uuid);

type Pool = r2d2_redis::r2d2::Pool<r2d2_redis::RedisConnectionManager>;
