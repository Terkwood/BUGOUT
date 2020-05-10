use super::AllEntryIds;
use super::RedisPool;
use redis_streams::repo::{fetch_entry_ids, update_entry_id};
use redis_streams::XReadEntryId;

use r2d2_redis::redis;
use std::collections::HashMap;
use std::sync::Arc;

pub trait EntryIdRepo {
    fn fetch_all(&self) -> Result<AllEntryIds, EidRepoErr>;

    fn update(&self, entry_id_type: EntryIdType, entry_id: XReadEntryId) -> Result<(), EidRepoErr>;
}

#[derive(Copy, Clone, Debug)]
pub enum EntryIdType {
    BotAttached,
    MoveMade,
}

#[derive(Debug)]
pub struct EidRepoErr;
impl From<redis::RedisError> for EidRepoErr {
    fn from(_: redis::RedisError) -> Self {
        EidRepoErr
    }
}

pub struct RedisEntryIdRepo {
    pool: Arc<RedisPool>,

    pub key_provider: super::KeyProvider,
}

const EMPTY_EID: &str = "0-0";
const MOVE_MADE_EID: &str = "move_made_eid";
const BOT_ATTACHED_EID: &str = "bot_attached_eid";

impl EntryIdRepo for RedisEntryIdRepo {
    fn fetch_all(&self) -> Result<AllEntryIds, EidRepoErr> {
        let redis_key = self.key_provider.entry_ids();
        let deser_hash: Box<dyn Fn(HashMap<String, String>) -> AllEntryIds> = Box::new(|f| {
            let bot_attached_eid = XReadEntryId::from_str(
                &f.get(BOT_ATTACHED_EID)
                    .unwrap_or(&EMPTY_EID.to_string())
                    .to_string(),
            )
            .unwrap_or(XReadEntryId::default());
            let move_made_eid = XReadEntryId::from_str(
                &f.get(MOVE_MADE_EID)
                    .unwrap_or(&EMPTY_EID.to_string())
                    .to_string(),
            )
            .unwrap_or(XReadEntryId::default());

            AllEntryIds {
                bot_attached_eid,
                move_made_eid,
            }
        });

        fetch_entry_ids(self.pool.as_ref(), &redis_key, deser_hash).map_err(|_| EidRepoErr)
    }

    fn update(&self, entry_id_type: EntryIdType, entry_id: XReadEntryId) -> Result<(), EidRepoErr> {
        let redis_key = self.key_provider.entry_ids();
        let hash_field = Box::new(|it| match it {
            EntryIdType::BotAttached => BOT_ATTACHED_EID.to_string(),
            EntryIdType::MoveMade => MOVE_MADE_EID.to_string(),
        });
        update_entry_id(entry_id_type, entry_id, &self.pool, &redis_key, hash_field)
            .map_err(|_| EidRepoErr)
    }
}

impl RedisEntryIdRepo {
    pub fn create_boxed(pool: Arc<RedisPool>) -> Box<dyn EntryIdRepo> {
        Box::new(RedisEntryIdRepo {
            pool,
            key_provider: super::KeyProvider::default(),
        })
    }
}
