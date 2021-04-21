mod components;
mod stream;

pub use components::Components;

/// it should listen to game states
///
/// on UndoMove:
///     it should emit a new game state
///     it should emit MoveUndone
fn _no() {}
