pub mod attached_bots;
pub mod board_size;
pub mod redis_keys;

pub use attached_bots::*;
pub use board_size::*;

#[derive(Debug)]
pub enum RepoErr {
    Redis(redis::RedisError),
    SomeErr,
    Conn,
}
impl From<redis::RedisError> for RepoErr {
    fn from(r: redis::RedisError) -> Self {
        RepoErr::Redis(r)
    }
}
