mod init;
mod process;
mod topics;
mod undo;
mod xack;
mod xadd;
mod xread;

pub use init::*;
pub use process::*;
pub use xack::*;
pub use xadd::*;
pub use xread::*;

use bot_model::api::*;
use move_model::GameState;
use undo_model::api::*;

pub const GROUP_NAME: &str = "undo";

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
