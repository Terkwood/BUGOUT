use super::xadd::XAdder;
use crossbeam_channel::Receiver;
use log::error;
use micro_model_bot::MoveComputed;
use std::sync::Arc;

pub fn write_moves(move_computed_out: Receiver<MoveComputed>, xadder: Arc<dyn XAdder>) {
    loop {
        // Block so that empty channel doesn't cause us to spin
        let msg = move_computed_out.recv();
        match msg {
            Ok(MoveComputed(command)) => {
                if let Err(e) = xadder.xadd_make_move_command(command) {
                    error!("could not xadd move command : {:?}", e)
                }
            }
            Err(e) => error!("recv: {}", e),
        }
    }
}
