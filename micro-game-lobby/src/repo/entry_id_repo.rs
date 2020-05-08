use redis_conn_pool::Pool;
use redis_streams::*;

use std::sync::Arc;

use super::FetchErr;

pub trait EntryIdRepo {
    fn fetch_all(&self) -> Result<AllEntryIds, FetchErr>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct AllEntryIds {
    pub find_public_game: XReadEntryId,
    pub create_game: XReadEntryId,
    pub join_private_game: XReadEntryId,
}

pub struct RedisEntryIdRepo {
    pub pool: Arc<Pool>,
    pub key_provider: (), // TODO
}

impl EntryIdRepo for RedisEntryIdRepo {
    fn fetch_all(&self) -> Result<AllEntryIds, FetchErr> {
        let mut conn = self.pool.get().unwrap();
        todo!()
    }
}
