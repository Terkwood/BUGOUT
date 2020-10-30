mod attachment;
mod board_size;
mod expire;

pub use attachment::*;
pub use board_size::*;
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
