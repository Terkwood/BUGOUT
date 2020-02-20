use std::collections::HashMap;
use uuid::Uuid;
#[derive(Debug, PartialEq)]
pub struct GameId(Uuid);

#[derive(Debug, PartialEq)]
pub struct ReqId(Uuid);
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Player {
    BLACK,
    WHITE,
}

#[derive(Debug)]
pub struct DeserError;
#[derive(Debug, PartialEq)]
pub struct MakeMoveCommand {
    pub game_id: GameId,
    pub req_id: ReqId,
    pub player: Player,
    pub coord: Option<Coord>,
}

impl MakeMoveCommand {
    pub fn from(xread_result: HashMap<String, String>) -> Result<MakeMoveCommand, DeserError> {
        println!("deser from {:#?}", xread_result);
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Coord {
    pub x: u16,
    pub y: u16,
}
