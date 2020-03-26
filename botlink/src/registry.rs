use crate::repo::{AttachedBotsRepo, EntryIdRepo, RedisAttachedBotsRepo, RedisEntryIdRepo};
use crate::stream::xadd::*;
use crate::stream::xread::{RedisXReader, XReader};
use crossbeam_channel::{unbounded, Receiver, Sender};
use micro_model_bot::{ComputeMove, MoveComputed};
use redis_conn_pool;
use redis_conn_pool::RedisHostUrl;
use std::sync::Arc;

pub struct Components {
    pub game_repo: Box<dyn AttachedBotsRepo>,
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub xreader: Box<dyn XReader>,
    pub xadder_gs: Box<dyn XAdderGS>,
    pub xadder_mm: Arc<dyn XAdderMM>,
    pub compute_move_in: Sender<ComputeMove>,
    pub compute_move_out: Receiver<ComputeMove>,
    pub move_computed_in: Sender<MoveComputed>,
    pub move_computed_out: Receiver<MoveComputed>,
}
impl Default for Components {
    fn default() -> Self {
        let (compute_move_in, compute_move_out): (Sender<ComputeMove>, Receiver<ComputeMove>) =
            unbounded();

        let (move_computed_in, move_computed_out): (Sender<MoveComputed>, Receiver<MoveComputed>) =
            unbounded();

        let pool = redis_conn_pool::create(RedisHostUrl::default());
        Components {
            game_repo: Box::new(RedisAttachedBotsRepo {
                pool: pool.clone(),
                key_provider: crate::repo::redis_keys::KeyProvider::default(),
            }),
            entry_id_repo: Box::new(RedisEntryIdRepo {
                pool: pool.clone(),
                key_provider: crate::repo::redis_keys::KeyProvider::default(),
            }),
            xreader: Box::new(RedisXReader { pool: pool.clone() }),
            xadder_gs: Box::new(RedisXAdderGS { pool: pool.clone() }),
            xadder_mm: Arc::new(RedisXAdderMM { pool }),
            compute_move_in,
            compute_move_out,
            move_computed_in,
            move_computed_out,
        }
    }
}
