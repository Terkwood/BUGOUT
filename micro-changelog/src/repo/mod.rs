pub mod game_states_repo;
pub mod redis_key;

use redis::RedisError;
use std::boxed::Box;

#[derive(Debug)]
pub enum WriteErr {
    Redis(RedisError),
    Serialization(std::boxed::Box<bincode::ErrorKind>),
    EIDRepo,
}
impl From<Box<bincode::ErrorKind>> for WriteErr {
    fn from(ek: Box<bincode::ErrorKind>) -> Self {
        WriteErr::Serialization(ek)
    }
}
impl From<RedisError> for WriteErr {
    fn from(r: RedisError) -> Self {
        WriteErr::Redis(r)
    }
}

#[derive(Debug)]
pub struct DeserError;
impl From<uuid::Error> for DeserError {
    fn from(_: uuid::Error) -> DeserError {
        DeserError
    }
}
impl From<std::num::ParseIntError> for DeserError {
    fn from(_: std::num::ParseIntError) -> DeserError {
        DeserError
    }
}

#[derive(Debug)]
pub enum FetchErr {
    Redis(redis::RedisError),
    Deser,
    EIDRepo,
}
impl From<DeserError> for FetchErr {
    fn from(_: DeserError) -> Self {
        FetchErr::Deser
    }
}
impl From<std::boxed::Box<bincode::ErrorKind>> for FetchErr {
    fn from(_: std::boxed::Box<bincode::ErrorKind>) -> Self {
        FetchErr::Deser
    }
}
impl From<redis::RedisError> for FetchErr {
    fn from(r: redis::RedisError) -> Self {
        FetchErr::Redis(r)
    }
}
