use super::*;
use crate::registry::Components;
use crate::repo::{AttachedBotsRepo, BoardSizeRepo};
use crossbeam_channel::Sender;
use micro_model_bot::ComputeMove;
use std::sync::Arc;

pub struct StreamOpts {
    pub attached_bots_repo: Box<dyn AttachedBotsRepo>,
    pub board_size_repo: Arc<dyn BoardSizeRepo>,
    pub xreader: Box<dyn xread::XReader>,
    pub xadder: Arc<dyn xadd::XAdder>,
    pub compute_move_in: Sender<ComputeMove>,
}

impl StreamOpts {
    pub fn from(components: Components) -> Self {
        StreamOpts {
            attached_bots_repo: components.ab_repo,
            board_size_repo: components.board_size_repo,
            xreader: components.xreader,
            xadder: components.xadder,
            compute_move_in: components.compute_move_in,
        }
    }
}
