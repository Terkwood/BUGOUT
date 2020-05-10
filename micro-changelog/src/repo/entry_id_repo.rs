use super::redis_key::ENTRY_IDS;
use super::{FetchErr, WriteErr};
use redis_streams::repo::{fetch_entry_ids, update_entry_id};
use redis_streams::*;

use std::collections::HashMap;
const GAME_READY_EID: &str = "game_ready_eid";
const GAME_STATES_EID: &str = "game_states_eid";
const MOVE_ACCEPTED_EID: &str = "move_accepted_eid";

use crate::Components;

pub fn fetch_all(components: &Components) -> Result<AllEntryIds, FetchErr> {
    let deser_hash: Box<dyn Fn(HashMap<String, String>) -> AllEntryIds> = Box::new(|f| {
        let game_ready_eid = XReadEntryId::from_str(
            &f.get(GAME_READY_EID)
                .unwrap_or(&XReadEntryId::default().to_string())
                .to_string(),
        )
        .unwrap_or(XReadEntryId::default());
        let game_states_eid = XReadEntryId::from_str(
            &f.get(GAME_STATES_EID)
                .unwrap_or(&XReadEntryId::default().to_string())
                .to_string(),
        )
        .unwrap_or(XReadEntryId::default());
        let move_accepted_eid = XReadEntryId::from_str(
            &f.get(MOVE_ACCEPTED_EID)
                .unwrap_or(&XReadEntryId::default().to_string())
                .to_string(),
        )
        .unwrap_or(XReadEntryId::default());
        AllEntryIds {
            game_ready_eid,
            game_states_eid,
            move_accepted_eid,
        }
    });
    fetch_entry_ids(&components.pool, ENTRY_IDS, deser_hash).map_err(|_| FetchErr::EIDRepo)
}
pub fn update(
    entry_id_type: EntryIdType,
    entry_id: XReadEntryId,
    components: &Components,
) -> Result<(), WriteErr> {
    let hash = Box::new(|entry_id_type| match entry_id_type {
        EntryIdType::GameStateChangelog => GAME_STATES_EID.to_string(),
        EntryIdType::GameReadyEvent => GAME_READY_EID.to_string(),
        EntryIdType::MoveAcceptedEvent => MOVE_ACCEPTED_EID.to_string(),
    });
    update_entry_id(entry_id_type, entry_id, &components.pool, ENTRY_IDS, hash)
        .map_err(|_| WriteErr::EIDRepo)
}

pub enum EntryIdType {
    GameReadyEvent,
    MoveAcceptedEvent,
    GameStateChangelog,
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
