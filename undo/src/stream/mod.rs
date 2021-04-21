mod topics;
mod xadd;

use bot_model::api::*;
use move_model::GameState;
use undo_model::api::*;
pub use xadd::*;

pub const GROUP_NAME: &str = "undo";

#[derive(Debug, Clone, PartialEq)]
pub enum StreamOutput {
    MU(MoveUndone),
    LOG(GameState),
}

#[derive(Clone, Debug)]
pub enum StreamInput {
    UM(UndoMove),
    LOG(GameState),
    BA(BotAttached),
}
