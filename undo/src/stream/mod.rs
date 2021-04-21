mod topics;
mod xadd;

use move_model::GameState;
use undo_model::api::*;
pub use xadd::*;

pub const GROUP_NAME: &str = "undo";

#[derive(Debug, Clone, PartialEq)]
pub enum StreamOutput {
    MU(MoveUndone),
    LOG(GameState),
}
