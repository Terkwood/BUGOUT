extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;

use micro_model_moves::*;
use serde_derive::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd)]
pub struct Id(pub String);

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd)]
pub struct Move(pub String, pub String);

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd)]
pub struct Rules(pub String);
impl Default for Rules {
    fn default() -> Self {
        Rules("tromp-taylor".to_string())
    }
}
#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub struct Komi(pub f32);
impl Default for Komi {
    fn default() -> Self {
        Komi(7.5)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
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

impl KataGoQuery {
    pub fn from(game_id: &GameId, game_state: &GameState) -> Self {
        KataGoQuery {
            id: Id(format!("{}_{}", game_id.0, game_state.turn)),
            ..KataGoQuery::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    #[test]
    fn query_from_game_state() {
        let game_id = GameId(Uuid::nil());
        let game_state = GameState {
            moves: vec![MoveMade {
                coord: Some(Coord::of(0, 0)),
                event_id: EventId::new(),
                game_id: game_id.clone(),
                reply_to: ReqId(Uuid::nil()),
                player: Player::BLACK,
                captured: vec![],
            }],
            ..GameState::default()
        };

        let expected = KataGoQuery {
            id: Id("00000000-0000-0000-0000-000000000000_1".to_string()),
            moves: vec![Move("B".to_string(), "A1".to_string())],
            ..KataGoQuery::default()
        };

        let actual = KataGoQuery::from(&game_id, &game_state);
        assert_eq!(actual, expected);
    }
}
