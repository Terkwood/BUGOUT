use super::AllEntryIds;
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::sync::Arc;
/*
#[derive(Copy, Clone, Debug)]
pub enum EntryIdType {
    BotAttached,
    MoveMade,
    HistProv,
    SyncReply,
    WaitOpponent,
    PrivGameReject,
    GameReady,
    ColorsChosen,
}
*/
 /*
 fn gen_xid(xid_name: &str, f: &HashMap<String, String>) -> XReadEntryId {
     XReadEntryId::from_str(
         &f.get(xid_name)
             .unwrap_or(&EMPTY_XID.to_string())
             .to_string(),
     )
     .unwrap_or(XReadEntryId::default())
 }*/
