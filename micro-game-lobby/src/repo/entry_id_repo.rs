use redis_streams::*;

use super::{FetchErr, RedisRepo, WriteErr};

pub trait EntryIdRepo {
    fn fetch_all(&self) -> Result<AllEntryIds, FetchErr>;
    fn update(&self, eid_type: EntryIdType, eid: XReadEntryId) -> Result<(), WriteErr>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct AllEntryIds {
    pub find_public_game: XReadEntryId,
    pub create_game: XReadEntryId,
    pub join_private_game: XReadEntryId,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum EntryIdType {
    FindPublicGameCmd,
    CreateGameCmd,
    JoinPrivateGameCmd,
}

impl EntryIdRepo for RedisRepo {
    fn fetch_all(&self) -> Result<AllEntryIds, FetchErr> {
        let _redis_key = self.key_provider.entry_ids();
        let _conn = self.pool.get().unwrap();
        todo!()
    }
    fn update(&self, _eid_type: EntryIdType, _eid: XReadEntryId) -> Result<(), WriteErr> {
        todo!()
    }
}
