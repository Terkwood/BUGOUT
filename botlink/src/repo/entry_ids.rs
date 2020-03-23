use redis_conn_pool::redis::Commands;
use redis_conn_pool::{redis, Pool};
use redis_streams::XReadEntryId;
use std::collections::HashMap;
pub trait EntryIdRepo {
    fn fetch_all(&self) -> Result<AllEntryIds, super::RepoErr>;

    fn update(
        &self,
        entry_id_type: EntryIdType,
        entry_id: XReadEntryId,
    ) -> Result<(), redis::RedisError>;
}

pub struct RedisEntryIdRepo {
    pub pool: Pool,
    pub key_provider: super::redis_keys::KeyProvider,
}
const EMPTY_EID: &str = "0-0";
impl EntryIdRepo for RedisEntryIdRepo {
    fn fetch_all(&self) -> Result<AllEntryIds, super::RepoErr> {
        let mut conn = self.pool.get().expect("pool");
        let found: Result<HashMap<String, String>, _> = conn.hgetall(self.key_provider.entry_ids());
        if let Ok(f) = found {
            let attach_bot_eid = XReadEntryId::from_str(
                &f.get(ATTACH_BOT_EID)
                    .unwrap_or(&EMPTY_EID.to_string())
                    .to_string(),
            )
            .unwrap_or(XReadEntryId::default());
            let game_states_eid = XReadEntryId::from_str(
                &f.get(GAME_STATES_EID)
                    .unwrap_or(&EMPTY_EID.to_string())
                    .to_string(),
            )
            .unwrap_or(XReadEntryId::default());
            Ok(AllEntryIds {
                game_states_eid,
                attach_bot_eid,
            })
        } else {
            Ok(AllEntryIds::default())
        }
    }
    fn update(&self, eid_type: EntryIdType, eid: XReadEntryId) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.get().expect("redis pool");
        conn.hset(
            self.key_provider.entry_ids(),
            eid_type.hash_field(),
            eid.to_string(),
        )
    }
}

pub enum EntryIdType {
    AttachBotEvent,
    GameStateChangelog,
}
const GAME_STATES_EID: &str = "game_states_eid";
const ATTACH_BOT_EID: &str = "attach_bot_eid";
impl EntryIdType {
    pub fn hash_field(&self) -> String {
        match self {
            EntryIdType::GameStateChangelog => GAME_STATES_EID.to_string(),
            EntryIdType::AttachBotEvent => ATTACH_BOT_EID.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AllEntryIds {
    pub attach_bot_eid: XReadEntryId,
    pub game_states_eid: XReadEntryId,
}
impl Default for AllEntryIds {
    fn default() -> Self {
        AllEntryIds {
            attach_bot_eid: XReadEntryId::default(),
            game_states_eid: XReadEntryId::default(),
        }
    }
}
