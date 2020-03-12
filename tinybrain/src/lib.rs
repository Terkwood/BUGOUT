extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use serde_derive::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Move(pub String, pub String);

#[derive(Debug, Clone, Serialize)]
pub struct Rules(pub String);
#[derive(Debug, Clone, Serialize)]
pub struct Komi(pub f32);

#[derive(Debug, Clone, Serialize)]
pub struct KataGoQuery {
    pub id: String,
    #[serde(rename = "initialStone")]
    pub initial_stones: Vec<Move>,
    pub moves: Vec<Move>,
    pub rules: Rules,
    pub komi: Komi,
    #[serde(rename = "boardXSize")]
    pub board_x_size: u16,
    #[serde(rename = "boardYSize")]
    pub board_y_size: u16,
}
