/// it should listen to game states
///
/// on UndoMove:
///     it should emit a new game state
///     it should emit MoveUndone
/// see also https://github.com/Terkwood/BUGOUT/issues/479
mod components;
mod repo;
pub mod stream;

pub use components::*;
