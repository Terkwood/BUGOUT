mod game_ready;
mod prefs;

pub use game_ready::*;
pub use prefs::*;

use crate::model::*;
use redis::{Client, Commands, Connection};
use std::rc::Rc;

#[derive(Debug)]
pub struct FetchErr;

#[derive(Debug)]
pub struct WriteErr;

const EXPIRY_SECS: usize = 86400;

pub fn touch_ttl(key: &str, conn: &mut Connection) -> Result<(), WriteErr> {
    let exp: Result<(), _> = conn.expire(key, EXPIRY_SECS).map_err(|_| WriteErr);

    exp
}

impl From<redis::RedisError> for FetchErr {
    fn from(_: redis::RedisError) -> Self {
        Self
    }
}
impl From<redis::RedisError> for WriteErr {
    fn from(_: redis::RedisError) -> Self {
        Self
    }
}
