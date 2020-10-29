mod attached_bots;
mod board_size;
mod difficulty;
mod expire;

pub use attached_bots::*;
pub use board_size::*;
pub use difficulty::*;
use expire::*;

#[derive(Debug)]
pub enum RepoErr {
    Redis(redis::RedisError),
    SerDes(Box<bincode::ErrorKind>),
    Conn,
}
impl From<redis::RedisError> for RepoErr {
    fn from(r: redis::RedisError) -> Self {
        RepoErr::Redis(r)
    }
}
impl From<Box<bincode::ErrorKind>> for RepoErr {
    fn from(e: Box<bincode::ErrorKind>) -> Self {
        RepoErr::SerDes(e)
    }
}
