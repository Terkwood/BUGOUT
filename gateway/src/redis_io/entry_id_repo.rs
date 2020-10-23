use super::AllEntryIds;
use redis_streams::repo::{fetch_entry_ids, update_entry_id};
use redis_streams::XReadEntryId;
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
    HistProv,
    SyncReply,
    WaitOpponent,
    PrivGameReject,
    GameReady,
    ColorsChosen,
}

#[derive(Debug)]
pub struct EidRepoErr;
impl From<redis::RedisError> for EidRepoErr {
    fn from(_: redis::RedisError) -> Self {
        EidRepoErr
    }
}

pub struct RedisEntryIdRepo {
    client: Arc<redis::Client>,

    pub key_provider: super::KeyProvider,
}

const EMPTY_XID: &str = "0-0";

const MOVE_MADE_XID: &str = "move_made_eid";
const BOT_ATTACHED_XID: &str = "bot_attached_eid";
const HIST_PROV_XID: &str = "hist_prov_eid";
const SYNC_REPLY_XID: &str = "sync_reply_eid";
const WAIT_OPPONENT_XID: &str = "wait_opponent_eid";
const GAME_READY_XID: &str = "game_ready_eid";
const PRIV_GAME_REJECT_XID: &str = "priv_game_reject_eid";
const COLORS_CHOSEN_XID: &str = "colors_chosen_eid";

impl EntryIdRepo for RedisEntryIdRepo {
    fn fetch_all(&self) -> Result<AllEntryIds, EidRepoErr> {
        let redis_key = self.key_provider.entry_ids();
        let deser_hash: Box<dyn Fn(HashMap<String, String>) -> AllEntryIds> = Box::new(|f| {
            let bot_attached_xid = gen_xid(BOT_ATTACHED_XID, &f);
            let move_made_xid = gen_xid(MOVE_MADE_XID, &f);
            let hist_prov_xid = gen_xid(HIST_PROV_XID, &f);
            let sync_reply_xid = gen_xid(SYNC_REPLY_XID, &f);
            let wait_opponent_xid = gen_xid(WAIT_OPPONENT_XID, &f);
            let game_ready_xid = gen_xid(GAME_READY_XID, &f);
            let priv_game_reject_xid = gen_xid(PRIV_GAME_REJECT_XID, &f);
            let colors_chosen_xid = gen_xid(COLORS_CHOSEN_XID, &f);

            AllEntryIds {
                bot_attached_xid,
                move_made_xid,
                hist_prov_xid,
                sync_reply_xid,
                wait_opponent_xid,
                game_ready_xid,
                priv_game_reject_xid,
                colors_chosen_xid,
            }
        });

        fetch_entry_ids(self.client.as_ref(), &redis_key, deser_hash).map_err(|_| EidRepoErr)
    }

    fn update(&self, entry_id_type: EntryIdType, entry_id: XReadEntryId) -> Result<(), EidRepoErr> {
        let redis_key = self.key_provider.entry_ids();
        let hash_field = Box::new(|it| match it {
            EntryIdType::BotAttached => BOT_ATTACHED_XID.to_string(),
            EntryIdType::MoveMade => MOVE_MADE_XID.to_string(),
            EntryIdType::HistProv => HIST_PROV_XID.to_string(),
            EntryIdType::SyncReply => SYNC_REPLY_XID.to_string(),
            EntryIdType::WaitOpponent => WAIT_OPPONENT_XID.to_string(),
            EntryIdType::GameReady => GAME_READY_XID.to_string(),
            EntryIdType::PrivGameReject => PRIV_GAME_REJECT_XID.to_string(),
            EntryIdType::ColorsChosen => COLORS_CHOSEN_XID.to_string(),
        });
        update_entry_id(
            entry_id_type,
            entry_id,
            &self.client,
            &redis_key,
            hash_field,
        )
        .map_err(|_| EidRepoErr)
    }
}
fn gen_xid(xid_name: &str, f: &HashMap<String, String>) -> XReadEntryId {
    XReadEntryId::from_str(
        &f.get(xid_name)
            .unwrap_or(&EMPTY_XID.to_string())
            .to_string(),
    )
    .unwrap_or(XReadEntryId::default())
}

impl RedisEntryIdRepo {
    pub fn create_boxed(pool: Arc<RedisPool>) -> Box<dyn EntryIdRepo> {
        Box::new(RedisEntryIdRepo {
            client: pool,
            key_provider: super::KeyProvider::default(),
        })
    }
}
