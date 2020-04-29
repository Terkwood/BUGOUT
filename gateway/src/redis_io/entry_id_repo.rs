use super::AllEntryIds;
use super::RedisPool;
use redis_streams::XReadEntryId;

use r2d2_redis::redis;
use redis::Commands;
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
impl EntryIdType {
    pub fn hash_field(&self) -> String {
        match self {
            EntryIdType::BotAttached => BOT_ATTACHED_EID.to_string(),
            EntryIdType::MoveMade => MOVE_MADE_EID.to_string(),
        }
    }
}
#[derive(Debug)]
pub enum EidRepoErr {
    Redis(redis::RedisError),
}
impl From<redis::RedisError> for EidRepoErr {
    fn from(r: redis::RedisError) -> Self {
        EidRepoErr::Redis(r)
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
        let mut conn = self.pool.get().expect("pool");
        let found: Result<HashMap<String, String>, _> = conn.hgetall(self.key_provider.entry_ids());
        let f = found?;
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

        Ok(AllEntryIds {
            bot_attached_eid,
            move_made_eid,
        })
    }

    fn update(&self, entry_id_type: EntryIdType, entry_id: XReadEntryId) -> Result<(), EidRepoErr> {
        let mut conn = self.pool.get().expect("redis pool");
        Ok(conn.hset(
            self.key_provider.entry_ids(),
            entry_id_type.hash_field(),
            entry_id.to_string(),
        )?)
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
