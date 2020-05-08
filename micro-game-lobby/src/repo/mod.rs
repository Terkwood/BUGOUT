mod entry_id_repo;
mod game_lobby_repo;

pub use entry_id_repo::*;
pub use game_lobby_repo::*;

use redis_conn_pool::Pool;
use std::sync::Arc;

#[derive(Debug)]
pub enum FetchErr {}

#[derive(Debug)]
pub struct WriteErr;

pub struct RedisRepo {
    pub pool: Arc<Pool>,
    pub key_provider: (), // TODO
}
