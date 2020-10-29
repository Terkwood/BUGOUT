pub mod attached_bots;
pub mod board_size;
mod difficulty;
pub mod redis_keys;

pub use attached_bots::*;
pub use board_size::*;
pub use difficulty::*;

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
