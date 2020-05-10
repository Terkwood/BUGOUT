use crate::io;
use io::conn_pool::Pool;
use io::r2d2_redis::redis;
use io::redis::Commands;
use io::redis_keys::*;
use io::FetchErr;
use redis_streams::repo::{fetch_entry_ids, update_entry_id};
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::rc::Rc;

const MAKE_MOVES_EID: &str = "make_moves_eid";
const GAME_STATES_EID: &str = "game_states_eid";
const EMPTY_EID: &str = "0-0";

#[derive(Clone)]
pub struct EntryIdRepo {
    pub namespace: RedisKeyNamespace,
    pub pool: Rc<Pool>,
}

impl EntryIdRepo {
    pub fn fetch_all(&self) -> Result<AllEntryIds, FetchErr> {
        let redis_key = entry_ids_hash_key(&self.namespace);
        let deser = Box::new(|f: HashMap<String, String>| {
            let make_moves_eid = f
                .get(MAKE_MOVES_EID)
                .unwrap_or(&EMPTY_EID.to_string())
                .to_string();
            let game_states_eid = f
                .get(GAME_STATES_EID)
                .unwrap_or(&EMPTY_EID.to_string())
                .to_string();
            AllEntryIds {
                make_moves_eid: XReadEntryId::from_str(&make_moves_eid).unwrap_or_default(),
                game_states_eid: XReadEntryId::from_str(&game_states_eid).unwrap_or_default(),
            }
        });
        fetch_entry_ids(&self.pool, &redis_key, deser).map_err(|_| FetchErr::EidRepo)
    }
    pub fn update(
        &self,
        entry_id_type: EntryIdType,
        entry_id: XReadEntryId,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.get().unwrap();

        conn.hset(
            entry_ids_hash_key(&self.namespace),
            entry_id_type.hash_field(),
            entry_id.to_string(),
        )
    }
}

pub enum EntryIdType {
    MakeMoveCommand,
    GameStateChangelog,
}
impl EntryIdType {
    pub fn hash_field(&self) -> String {
        match self {
            EntryIdType::GameStateChangelog => GAME_STATES_EID.to_string(),
            EntryIdType::MakeMoveCommand => MAKE_MOVES_EID.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AllEntryIds {
    pub make_moves_eid: XReadEntryId,
    pub game_states_eid: XReadEntryId,
}
impl Default for AllEntryIds {
    fn default() -> Self {
        AllEntryIds {
            make_moves_eid: XReadEntryId::default(),
            game_states_eid: XReadEntryId::default(),
        }
    }
}
