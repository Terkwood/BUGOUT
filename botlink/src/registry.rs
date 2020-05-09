use crate::repo::*;
use crate::stream::xadd::*;
use crate::stream::xread::{RedisXReader, XReader};
use crossbeam_channel::{unbounded, Receiver, Sender};
use micro_model_bot::{ComputeMove, MoveComputed};
use redis_conn_pool;
use redis_conn_pool::RedisHostUrl;
use std::sync::Arc;

pub struct Components {
    pub ab_repo: Box<dyn AttachedBotsRepo>,
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub board_size_repo: Arc<dyn BoardSizeRepo>,
    pub xreader: Box<dyn XReader>,
    pub xadder: Arc<dyn XAdder>,
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

        let pool = Arc::new(redis_conn_pool::create(RedisHostUrl::default()));
        Components {
            ab_repo: Box::new(pool.clone()),
            entry_id_repo: Box::new(pool.clone()),
            board_size_repo: Arc::new(pool.clone()),
            xreader: Box::new(RedisXReader { pool: pool.clone() }),
            xadder: Arc::new(RedisXAdder { pool }),
            compute_move_in,
            compute_move_out,
            move_computed_in,
            move_computed_out,
        }
    }
}
