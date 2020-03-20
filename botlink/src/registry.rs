use crate::repo::entry_id::{EntryIdRepo, RedisEntryIdRepo};
use crate::repo::game::{GameRepo, RedisGameRepo};
use crate::stream::xread::{RedisXReader, XReader};
use crossbeam_channel::{unbounded, Receiver, Sender};
use micro_model_bot::{ComputeMove, MoveComputed};
use redis_conn_pool;
use redis_conn_pool::RedisHostUrl;

pub struct Components {
    pub game_repo: Box<dyn GameRepo>,
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub xreader: Box<dyn XReader>,
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
            game_repo: Box::new(RedisGameRepo { pool: pool.clone() }),
            entry_id_repo: Box::new(RedisEntryIdRepo {
                pool: pool.clone(),
                key_provider: crate::repo::redis_keys::KeyProvider::default(),
            }),
            xreader: Box::new(RedisXReader { pool }),
            compute_move_in,
            compute_move_out,
            move_computed_in,
            move_computed_out,
        }
    }
}