use redis::Client;
use redis_streams::repo::{fetch_entry_ids, update_entry_id};
use redis_streams::*;
use std::collections::HashMap;
use std::rc::Rc;

use super::ENTRY_ID_KEY;
use super::{FetchErr, WriteErr};

pub trait EntryIdRepo {
    fn fetch_all(&self) -> Result<AllEntryIds, FetchErr>;
    fn update(&self, eid_type: EntryIdType, eid: XReadEntryId) -> Result<(), WriteErr>;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AllEntryIds {
    pub find_public_game: XReadEntryId,
    pub create_game: XReadEntryId,
    pub join_private_game: XReadEntryId,
    pub session_disconnected: XReadEntryId,
}
impl Default for AllEntryIds {
    fn default() -> Self {
        AllEntryIds {
            find_public_game: XReadEntryId::default(),
            create_game: XReadEntryId::default(),
            join_private_game: XReadEntryId::default(),
            session_disconnected: XReadEntryId::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum EntryIdType {
    FindPublicGameCmd,
    CreateGameCmd,
    JoinPrivateGameCmd,
    SessionDisconnectedEv,
}

const FIND_PUBLIC_EID: &str = "find_public";
const CREATE_GAME_EID: &str = "create";
const JOIN_PRIVATE_EID: &str = "join_private";
const SESSION_DISCONN_EID: &str = "session_disconn";

impl EntryIdRepo for Rc<Client> {
    fn fetch_all(&self) -> Result<AllEntryIds, FetchErr> {
        let deser_hash: Box<dyn Fn(HashMap<String, String>) -> AllEntryIds> =
            Box::new(|f| AllEntryIds {
                find_public_game: lookup(&f, FIND_PUBLIC_EID),
                create_game: lookup(&f, CREATE_GAME_EID),
                join_private_game: lookup(&f, JOIN_PRIVATE_EID),
                session_disconnected: lookup(&f, SESSION_DISCONN_EID),
            });
        fetch_entry_ids(&*self, ENTRY_ID_KEY, deser_hash).map_err(|_| FetchErr)
    }
    fn update(&self, eid_type: EntryIdType, eid: XReadEntryId) -> Result<(), WriteErr> {
        let hash_field = Box::new(|eid_type| {
            match eid_type {
                EntryIdType::FindPublicGameCmd => FIND_PUBLIC_EID,
                EntryIdType::CreateGameCmd => CREATE_GAME_EID,
                EntryIdType::JoinPrivateGameCmd => JOIN_PRIVATE_EID,
                EntryIdType::SessionDisconnectedEv => SESSION_DISCONN_EID,
            }
            .to_string()
        });
        update_entry_id(eid_type, eid, &*self, ENTRY_ID_KEY, hash_field).map_err(|_| WriteErr)
    }
}

/// Looks up the EID corresponding to a given field in a hash
/// return by redis.
/// for example, you might look up an entry ID of "1000-0"
/// for the create games stream.
fn lookup(hash: &HashMap<String, String>, eid_field: &str) -> XReadEntryId {
    XReadEntryId::from_str(
        &hash
            .get(eid_field)
            .unwrap_or(&XReadEntryId::default().to_string())
            .to_string(),
    )
    .unwrap_or(XReadEntryId::default())
}
