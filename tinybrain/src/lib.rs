extern crate base64;
extern crate bincode;
extern crate dotenv;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate tungstenite;
extern crate uuid;
#[macro_use]
extern crate lazy_static;
extern crate http;

pub mod env;
mod err;
pub mod katago;
pub mod websocket;

use katago::json::KataGoResponse;
use micro_model_moves::*;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeMove {
    pub game_id: GameId,
    pub game_state: GameState,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoveComputed(MakeMoveCommand);

impl MoveComputed {
    pub fn from(response: KataGoResponse) -> Result<Self, err::KataGoParseErr> {
        let game_id = response.game_id()?;
        let player = response.player()?;
        let coord: Option<Coord> = interpret_coord(&response.move_infos[0].r#move)?;
        let req_id = ReqId(Uuid::new_v4());
        Ok(MoveComputed(MakeMoveCommand {
            game_id,
            player,
            coord,
            req_id,
        }))
    }
}

fn interpret_coord(move_info_move: &str) -> Result<Option<Coord>, err::CoordOutOfRange> {
    if move_info_move.trim().to_ascii_lowercase() == katago::json::PASS {
        Ok(None)
    } else {
        Ok(Some(from_alphanum(move_info_move)?))
    }
}

fn from_alphanum(a: &str) -> Result<Coord, err::CoordOutOfRange> {
    if a.len() < 2 {
        Err(err::CoordOutOfRange)
    } else {
        let letter: char = a.chars().collect::<Vec<char>>()[0];
        let number = &a[1..];
        let y_plus_one = number.to_string().parse::<u16>()?;
        let r: Vec<char> = (b'A'..=b'Z').map(char::from).collect();
        let maybe_x = r.iter().position(|l| l == &letter);
        if let Some(x) = maybe_x {
            Ok(Coord {
                x: x as u16,
                y: y_plus_one - 1,
            })
        } else {
            Err(err::CoordOutOfRange)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use katago::json::*;
    use uuid::Uuid;
    #[test]
    fn move_computed_from() {
        let actual = MoveComputed::from(KataGoResponse {
            id: Id(format!("{}_1_WHITE", Uuid::nil().to_string())),
            turn_number: 1,
            move_infos: vec![MoveInfo {
                r#move: "B3".to_string(),
                order: 0,
            }],
        })
        .expect("fail");
        let expected = MoveComputed(MakeMoveCommand {
            game_id: GameId(Uuid::nil()),
            coord: Some(Coord { x: 1, y: 2 }),
            player: Player::WHITE,
            req_id: actual.0.req_id.clone(),
        });
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_interpret_coord() {
        let actual = interpret_coord("B3");
        assert_eq!(actual.expect("parse"), Some(Coord { x: 1, y: 2 }))
    }

    #[test]
    fn test_interpret_pass() {
        let actual = interpret_coord("pass");
        assert_eq!(actual.expect("parse"), None)
    }
}
