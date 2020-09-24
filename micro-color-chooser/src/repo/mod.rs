mod game;
mod prefs;

pub use game::*;
pub use prefs::*;

use crate::model::*;
use redis::Client;
use std::rc::Rc;

pub struct FetchErr;
pub struct WriteErr;

const EXPIRY_SECS: usize = 86400;

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
