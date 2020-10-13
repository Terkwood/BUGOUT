pub mod entry_id_repo;
mod into_shared;
mod key_provider;
mod namespace;
pub mod stream;
pub mod xadd;
pub mod xread;

pub use key_provider::*;
pub use namespace::*;
pub use xadd::start;

use r2d2_redis::{r2d2, RedisConnectionManager};
use redis_streams::XReadEntryId;
use std::sync::Arc;

pub type RedisPool = r2d2_redis::r2d2::Pool<r2d2_redis::RedisConnectionManager>;

pub const REDIS_URL: &str = "redis://redis";

pub fn create_pool() -> Arc<RedisPool> {
    let manager = RedisConnectionManager::new(REDIS_URL).unwrap();
    Arc::new(r2d2::Pool::builder().build(manager).unwrap())
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
