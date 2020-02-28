use super::redis_key::*;
use super::{FetchErr, WriteErr};
use crate::redis;
use micro_model_moves::{GameId, GameState};
use redis_conn_pool::redis::Commands;
use redis_conn_pool::{Pool, RedisHostUrl};
use redis_streams::*;
use std::collections::HashMap;
const GAME_READY_EID: &str = "game_ready_eid";
const GAME_STATES_EID: &str = "game_states_eid";
const MOVE_ACCEPTED_EID: &str = "move_accepted_eid";
const EMPTY_EID: &str = "0-0";

#[derive(Clone)]
pub struct EntryIdRepo {
    pub namespace: RedisKeyNamespace,
    pub pool: Pool,
    pub hash_key_provider: HashKeyProvider,
}
impl EntryIdRepo {
    pub fn fetch_all(&self) -> Result<AllEntryIds, FetchErr> {
        let mut conn = self.pool.get().unwrap();
        let found: Result<HashMap<String, String>, _> =
            conn.hgetall(self.hash_key_provider.entry_ids());
        if let Ok(f) = found {
            let game_ready_eid = XReadEntryId::from_str(
                &f.get(GAME_READY_EID)
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
            let move_accepted_eid = XReadEntryId::from_str(
                &f.get(MOVE_ACCEPTED_EID)
                    .unwrap_or(&EMPTY_EID.to_string())
                    .to_string(),
            )
            .unwrap_or(XReadEntryId::default());
            Ok(AllEntryIds {
                game_ready_eid,
                game_states_eid,
                move_accepted_eid,
            })
        } else {
            Ok(AllEntryIds::default())
        }
    }
    pub fn update(
        &self,
        entry_id_type: EntryIdType,
        entry_id: XReadEntryId,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.get().unwrap();
        conn.hset(
            self.hash_key_provider.entry_ids(),
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
            EntryIdType::MakeMoveCommand => GAME_READY_EID.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AllEntryIds {
    pub game_ready_eid: XReadEntryId,
    pub move_accepted_eid: XReadEntryId,
    pub game_states_eid: XReadEntryId,
}
impl Default for AllEntryIds {
    fn default() -> Self {
        AllEntryIds {
            game_ready_eid: XReadEntryId::default(),
            move_accepted_eid: XReadEntryId::default(),
            game_states_eid: XReadEntryId::default(),
        }
    }
}
