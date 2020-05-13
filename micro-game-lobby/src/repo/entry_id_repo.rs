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

impl EntryIdRepo for Rc<Client> {
    fn fetch_all(&self) -> Result<AllEntryIds, FetchErr> {
        let deser_hash: Box<dyn Fn(HashMap<String, String>) -> AllEntryIds> =
            Box::new(|_f| todo!());
        fetch_entry_ids(&*self, ENTRY_ID_KEY, deser_hash).map_err(|_| FetchErr::EIDRepo)
    }
    fn update(&self, eid_type: EntryIdType, eid: XReadEntryId) -> Result<(), WriteErr> {
        let hash_field = Box::new(|eid_type| match eid_type {
            EntryIdType::FindPublicGameCmd => todo!(),
            EntryIdType::CreateGameCmd => todo!(),
            EntryIdType::JoinPrivateGameCmd => todo!(),
            EntryIdType::SessionDisconnectedEv => todo!(),
        });
        update_entry_id(eid_type, eid, &*self, ENTRY_ID_KEY, hash_field)
            .map_err(|_| WriteErr::EIDRepo)
    }
}
