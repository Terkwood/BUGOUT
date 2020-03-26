use super::xadd::XAdderMM;
use crossbeam_channel::{select, Receiver};
use log::error;
use micro_model_bot::MoveComputed;
use std::sync::Arc;
pub fn write_moves(move_computed_out: Receiver<MoveComputed>, xadder: Arc<dyn XAdderMM>) {
    loop {
        select! {
            recv(move_computed_out) -> msg => match msg {
                Ok(MoveComputed(command)) => if let Err(e)=xadder.xadd_make_move_command(command) {error!("could not xadd move command : {:?}",e)},
                Err(e) => error!("Unable to receive move computed out {:?}",e)
            }
        }
    }
}
