extern crate core_model;
extern crate move_model;
extern crate serde_derive;

pub mod api;

use serde_derive::{Deserialize, Serialize};

use move_model::{Coord, Player};
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Move {
    pub player: Player,
    pub coord: Option<Coord>,
    pub turn: u32,
}
