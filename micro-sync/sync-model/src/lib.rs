pub use core_model;
pub use move_model;

pub mod api;

use serde_derive::{Deserialize, Serialize};

use move_model::{Coord, Player};
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct Move {
    pub player: Player,
    pub coord: Option<Coord>,
    pub turn: u32,
}
