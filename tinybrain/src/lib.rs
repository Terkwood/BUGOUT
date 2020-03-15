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
        let coord: Option<Coord> = katago::json::interpret_coord(&response.move_infos[0].r#move)?;
        let req_id = ReqId(Uuid::new_v4());
        Ok(MoveComputed(MakeMoveCommand {
            game_id,
            player,
            coord,
            req_id,
        }))
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
}
