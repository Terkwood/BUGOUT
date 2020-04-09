use super::xadd::XAdder;
use crossbeam_channel::{select, Receiver};
use log::error;
use micro_model_bot::MoveComputed;
use std::sync::Arc;
pub fn write_moves(move_computed_out: Receiver<MoveComputed>, xadder: Arc<dyn XAdder>) {
    loop {
        select! { recv(move_computed_out) -> msg =>
            if let Ok(MoveComputed(command)) = msg {
                if let Err(e)=xadder.xadd_make_move_command(command) {
                    error!("could not xadd move command : {:?}",e)
                }
            } // swallow RecvError since this channel may be empty at startup
        }
    }
}
