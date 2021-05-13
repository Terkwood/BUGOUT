mod handlers;
mod topics;
mod undo;
mod xadd;
mod xread;

pub use handlers::init;
pub use xadd::*;
pub use xread::*;

use bot_model::api::*;
use move_model::GameState;
use undo_model::api::*;


#[derive(Debug, Clone, PartialEq)]
pub enum StreamOutput {
    MU(MoveUndone),
    LOG(GameState),
    REJECT(UndoMove),
}

#[derive(Clone, Debug)]
pub enum StreamInput {
    UM(UndoMove),
    LOG(GameState),
    BA(BotAttached),
}
