use redis_streams::*;

use super::{FetchErr, RedisRepo};

pub trait EntryIdRepo {
    fn fetch_all(&self) -> Result<AllEntryIds, FetchErr>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct AllEntryIds {
    pub find_public_game: XReadEntryId,
    pub create_game: XReadEntryId,
    pub join_private_game: XReadEntryId,
}

impl EntryIdRepo for RedisRepo {
    fn fetch_all(&self) -> Result<AllEntryIds, FetchErr> {
        let _redis_key = self.key_provider.entry_ids();
        let _conn = self.pool.get().unwrap();
        todo!()
    }
}
