mod history;
mod reply;

pub use history::*;
pub use reply::*;

use log::error;
use redis::{Commands, Connection, RedisError};

const EXPIRY_SECS: usize = 86400;

/// update a record's ttl. will never fail the calling
/// function, but it will write to error log
/// if there's a problem
pub fn touch_ttl(conn: &mut Connection, key: &str) {
    let exp: Result<(), _> = conn.expire(key, EXPIRY_SECS);
    if let Err(e) = exp {
        error!("touch TTL error {:?}", e)
    }
}

#[derive(Debug)]
pub enum FetchErr {
    Deser(Box<bincode::ErrorKind>),
    Redis(RedisError),
}
pub struct WriteErr;

impl From<redis::RedisError> for FetchErr {
    fn from(e: redis::RedisError) -> Self {
        FetchErr::Redis(e)
    }
}
impl From<redis::RedisError> for WriteErr {
    fn from(_: redis::RedisError) -> Self {
        Self
    }
}
