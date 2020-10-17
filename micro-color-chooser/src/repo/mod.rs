mod game_ready;
mod prefs;

pub use game_ready::*;
pub use prefs::*;

use color_model::*;
use log::error;
use redis::{Client, Commands, Connection, RedisError};
use std::rc::Rc;

#[derive(Debug)]
pub enum FetchErr {
    Redis(RedisError),
    Deser(Box<bincode::ErrorKind>),
}

#[derive(Debug)]
pub struct WriteErr;

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
impl From<Box<bincode::ErrorKind>> for FetchErr {
    fn from(e: Box<bincode::ErrorKind>) -> Self {
        FetchErr::Deser(e)
    }
}
