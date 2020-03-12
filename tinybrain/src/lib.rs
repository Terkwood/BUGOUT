extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;

use serde_derive::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Id(pub String);

#[derive(Debug, Clone, Serialize)]
pub struct Move(pub String, pub String);

#[derive(Debug, Clone, Serialize)]
pub struct Rules(pub String);
impl Default for Rules {
    fn default() -> Self {
        Rules("tromp-taylor".to_string())
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Komi(pub f32);
impl Default for Komi {
    fn default() -> Self {
        Komi(7.5)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KataGoQuery {
    pub id: Id,
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

const DEFAULT_BOARD_SIZE: u16 = 19;
impl Default for KataGoQuery {
    fn default() -> Self {
        KataGoQuery {
            id: Id(uuid::Uuid::new_v4().to_string()),
            initial_stones: vec![],
            moves: vec![],
            komi: Komi::default(),
            rules: Rules::default(),
            board_x_size: DEFAULT_BOARD_SIZE,
            board_y_size: DEFAULT_BOARD_SIZE,
        }
    }
}
