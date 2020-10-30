use super::*;
use crate::registry::Components;
use crate::repo::{AttachmentRepo, BoardSizeRepo};
use bot_model::api::ComputeMove;
use crossbeam_channel::Sender;
use std::sync::Arc;

pub struct StreamOpts {
    pub attachment_repo: Box<dyn AttachmentRepo>,
    pub board_size_repo: Arc<dyn BoardSizeRepo>,
    pub xread: Box<dyn xread::XReader>,
    pub xadd: Arc<dyn xadd::XAdder>,
    pub xack: Arc<dyn xack::XAck>,
    pub compute_move_in: Sender<ComputeMove>,
}

impl StreamOpts {
    pub fn from(components: Components) -> Self {
        StreamOpts {
            attachment_repo: components.attachment_repo,
            board_size_repo: components.board_size_repo,
            xread: components.xreader,
            xadd: components.xadder,
            xack: components.xack,
            compute_move_in: components.compute_move_in,
        }
    }
}
