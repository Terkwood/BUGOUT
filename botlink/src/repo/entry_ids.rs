use super::redis_keys::ENTRY_IDS;
use super::RepoErr;
use redis_conn_pool::Pool;
use redis_streams::repo::{fetch_entry_ids, update_entry_id};
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::sync::Arc;
pub trait EntryIdRepo: Send + Sync {
    fn fetch_all(&self) -> Result<AllEntryIds, RepoErr>;

    fn update(&self, entry_id_type: EntryIdType, entry_id: XReadEntryId) -> Result<(), RepoErr>;
}

impl EntryIdRepo for Arc<Pool> {
    fn fetch_all(&self) -> Result<AllEntryIds, RepoErr> {
        let deser_hash: Box<dyn Fn(HashMap<String, String>) -> AllEntryIds> = Box::new(|hash| {
            let attach_bot_eid = XReadEntryId::from_str(
                &hash
                    .get(ATTACH_BOT_EID)
                    .unwrap_or(&XReadEntryId::default().to_string())
                    .to_string(),
            )
            .unwrap_or(XReadEntryId::default());
            let game_states_eid = XReadEntryId::from_str(
                &hash
                    .get(GAME_STATES_EID)
                    .unwrap_or(&XReadEntryId::default().to_string())
                    .to_string(),
            )
            .unwrap_or(XReadEntryId::default());
            AllEntryIds {
                game_states_eid,
                attach_bot_eid,
            }
        });
        let fetched = fetch_entry_ids(&self, ENTRY_IDS, deser_hash);

        fetched.map_err(|_| super::RepoErr::SomeErr)
    }
    fn update(&self, eid_type: EntryIdType, eid: XReadEntryId) -> Result<(), super::RepoErr> {
        let hash = Box::new(|eid_type| match eid_type {
            EntryIdType::GameStateChangelog => GAME_STATES_EID.to_string(),
            EntryIdType::AttachBotEvent => ATTACH_BOT_EID.to_string(),
        });
        update_entry_id(eid_type, eid, self, ENTRY_IDS, hash).map_err(|_| RepoErr::SomeErr)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EntryIdType {
    AttachBotEvent,
    GameStateChangelog,
}
const GAME_STATES_EID: &str = "game_states_eid";
const ATTACH_BOT_EID: &str = "attach_bot_eid";

#[derive(Debug, PartialEq, Eq, Clone)]
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
