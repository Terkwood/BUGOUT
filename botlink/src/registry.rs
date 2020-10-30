use crate::repo::*;
use crate::stream::xack::XAck;
use crate::stream::xadd::*;
use crate::stream::xread::XReader;
use bot_model::api::{ComputeMove, MoveComputed};
use crossbeam_channel::{unbounded, Receiver, Sender};
use redis::Client;
use std::sync::Arc;

pub struct Components {
    pub board_size_repo: Arc<dyn BoardSizeRepo>,
    pub attachment_repo: Box<dyn AttachmentRepo>,
    pub xreader: Box<dyn XReader>,
    pub xadder: Arc<dyn XAdder>,
    pub xack: Arc<dyn XAck>,
    pub compute_move_in: Sender<ComputeMove>,
    pub compute_move_out: Receiver<ComputeMove>,
    pub move_computed_in: Sender<MoveComputed>,
    pub move_computed_out: Receiver<MoveComputed>,
}

const REDIS_URL: &str = "redis://redis/";
pub fn create_redis_client() -> Arc<redis::Client> {
    Arc::new(Client::open(REDIS_URL).expect("redis client"))
}

impl Components {
    pub fn new(client: Arc<Client>) -> Self {
        let (compute_move_in, compute_move_out): (Sender<ComputeMove>, Receiver<ComputeMove>) =
            unbounded();

        let (move_computed_in, move_computed_out): (Sender<MoveComputed>, Receiver<MoveComputed>) =
            unbounded();

        Components {
            attachment_repo: Box::new(client.clone()),
            board_size_repo: Arc::new(client.clone()),
            xreader: Box::new(client.clone()),
            xadder: Arc::new(client.clone()),
            xack: Arc::new(client),
            compute_move_in,
            compute_move_out,
            move_computed_in,
            move_computed_out,
        }
    }
}
