use super::XReadEntryId;
use redis_conn_pool::redis::Commands;
use redis_conn_pool::Pool;
use std::collections::HashMap;
use std::sync::Arc;

pub trait AllEntryIds: Send + Sync {
    fn from_hash(
        &self,
        hash: HashMap<String, String>,
    ) -> Result<Box<dyn AllEntryIds>, EntryIdRepoErr>;
}

pub trait EntryIdType {}

pub trait EIDRepoKeyProvider: Send + Sync {
    fn entry_id_repo_key(&self) -> String;
}

pub trait AllEIDsDeser: Send + Sync {
    fn from_hash(
        &self,
        hash: HashMap<String, String>,
    ) -> Result<Box<dyn AllEntryIds, EntryIdRepoErr>;
}

pub trait EntryIdRepo: Send + Sync
where
{
    fn fetch_all(&self) -> Result<Box<dyn AllEntryIds>, EntryIdRepoErr>;
    fn update(
        &self,
        entry_id_type: Box<dyn EntryIdType>,
        entry_id: XReadEntryId,
    ) -> Result<(), EntryIdRepoErr>;
}

pub struct RepoContext {
    pub pool: Arc<Pool>,
    pub key_provider: Box<dyn EIDRepoKeyProvider>,
    pub deser_all_eids: Box<dyn AllEIDsDeser>,
    pub default_set: Box<dyn AllEntryIds>,
}

impl EntryIdRepo for RepoContext {
    fn fetch_all(&self) -> Result<Box<dyn AllEntryIds>, EntryIdRepoErr> {
        if let Ok(mut conn) = self.pool.get() {
            let found: Result<HashMap<String, String>, _> =
                conn.hgetall(self.key_provider.entry_id_repo_key());
            if let Ok(hash) = found {
                self.deser_all_eids.from_hash(hash)
            } else {
                Ok(self.default_set)
            }
        } else {
            Err(EntryIdRepoErr)
        }
    }
    fn update(
        &self,
        entry_id_type: Box<dyn EntryIdType>,
        entry_id: XReadEntryId,
    ) -> Result<(), EntryIdRepoErr> {
        todo!()
    }
}

pub struct EntryIdRepoErr;
