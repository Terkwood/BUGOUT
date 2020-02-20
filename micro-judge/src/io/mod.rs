pub mod conn_pool;
pub mod entry_id;
mod game_states_repo;
pub mod redis_keys;

pub mod stream;
pub mod topics;
mod xread;

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
}
impl From<DeserError> for FetchErr {
    fn from(_: DeserError) -> Self {
        FetchErr::Deser
    }
}
impl From<std::boxed::Box<bincode::ErrorKind>> for FetchErr {
    fn from(b: std::boxed::Box<bincode::ErrorKind>) -> Self {
        FetchErr::Deser
    }
}
impl From<redis::RedisError> for FetchErr {
    fn from(r: redis::RedisError) -> Self {
        FetchErr::Redis(r)
    }
}
