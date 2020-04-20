use super::xadd::XAdder;
use crossbeam_channel::{select, Receiver};
use log::{error, info};
use micro_model_bot::MoveComputed;
use std::sync::Arc;

pub fn write_moves(move_computed_out: Receiver<MoveComputed>, xadder: Arc<dyn XAdder>) {
    loop {
        select! { recv(move_computed_out) -> msg =>
            match msg {
                Ok(MoveComputed(command)) =>
                    if let Err(e) = xadder.xadd_make_move_command(&command) {
                        error!("could not xadd move command : {:?}",e)
                    } else {
                        info!("🆗 {:?}", command)
                    }
                Err(e) =>
                    error!("loop recv: {}", e)
            }
        }
    }
}
