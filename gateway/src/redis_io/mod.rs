mod key_provider;
mod namespace;
pub mod stream;
pub mod xadd;
pub mod xread;

pub use key_provider::*;
pub use namespace::*;
pub use xadd::start;

use redis::Client;
use redis_streams::XReadEntryId;
use std::sync::Arc;

pub const REDIS_URL: &str = "redis://redis";

pub fn create_redis_client() -> Arc<Client> {
    Arc::new(Client::open(REDIS_URL).expect("redis client"))
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AllEntryIds {
    pub bot_attached_xid: XReadEntryId,
    pub move_made_xid: XReadEntryId,
    pub hist_prov_xid: XReadEntryId,
    pub sync_reply_xid: XReadEntryId,
    pub wait_opponent_xid: XReadEntryId,
    pub game_ready_xid: XReadEntryId,
    pub priv_game_reject_xid: XReadEntryId,
    pub colors_chosen_xid: XReadEntryId,
}

impl Default for AllEntryIds {
    fn default() -> Self {
        AllEntryIds {
            bot_attached_xid: XReadEntryId::default(),
            move_made_xid: XReadEntryId::default(),
            hist_prov_xid: XReadEntryId::default(),
            sync_reply_xid: XReadEntryId::default(),
            wait_opponent_xid: XReadEntryId::default(),
            game_ready_xid: XReadEntryId::default(),
            priv_game_reject_xid: XReadEntryId::default(),
            colors_chosen_xid: XReadEntryId::default(),
        }
    }
}
