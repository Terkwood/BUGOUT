use super::xadd::XAdder;
use crate::repo::board_size::BoardSizeRepo;
use crossbeam_channel::{select, Receiver};
use log::{error, info};
use micro_model_bot::MoveComputed;
use std::sync::Arc;

pub fn write_moves(
    move_computed_out: Receiver<MoveComputed>,
    xadder: Arc<dyn XAdder>,
    board_size_repo: Arc<dyn BoardSizeRepo>,
) {
    loop {
        select! { recv(move_computed_out) -> msg =>
            match msg {
                Ok(MoveComputed { game_id, player, alphanum_coord }) => {
                    let command = todo!("look up board size and convert y appropriately");

                    if let Err(e) = xadder.xadd_make_move_command(&command) {
                        error!("could not xadd move command : {:?}",e)
                    } else {
                        info!("🆗 {:?}", command)
                    }
                }
                Err(e) =>
                    error!("loop recv: {}", e)
            }
        }
    }
}
