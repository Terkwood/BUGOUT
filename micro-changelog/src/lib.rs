extern crate bincode;
pub extern crate micro_model_moves;
pub mod repo;
pub mod stream;

pub use redis_conn_pool;
pub use redis_conn_pool::{r2d2, r2d2_redis, redis};
