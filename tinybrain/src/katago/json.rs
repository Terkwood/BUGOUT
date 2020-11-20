use crate::err::*;

use bot_model::api::ComputeMove;
use core_model::*;
use move_model::*;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

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
    #[serde(rename = "maxVisits", skip_serializing_if = "Option::is_none")]
    pub max_visits: Option<u16>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, PartialOrd)]
pub struct KataGoResponse {
    pub id: Id,
    #[serde(rename = "moveInfos")]
    pub move_infos: Vec<MoveInfo>,
    #[serde(rename = "turnNumber")]
    pub turn_number: u32,
}

/// In the form of
/// `GAMEID_BUGOUTTURN_WHOMOVED`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
pub struct Id(pub String);

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd)]
pub struct Move(pub String, pub KataCoordOrPass);

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd)]
pub struct Rules(pub String);

#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub struct Komi(pub f32);

#[derive(Debug, Clone, Deserialize, PartialEq, PartialOrd)]
pub struct MoveInfo {
    pub order: u32,
    pub r#move: String,
}

pub const PASS: &str = "pass";

/// Represent coords as (0,13) or PASS.
/// See https://github.com/lightvector/KataGo/blob/master/docs/Analysis_Engine.md#queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
pub struct KataCoordOrPass(pub String);

impl Move {
    pub fn from(player: Player, maybe_xy: Option<Coord>) -> Result<Self, CoordOutOfRange> {
        let p = match player {
            Player::BLACK => "B",
            _ => "W",
        };
        KataCoordOrPass::from(maybe_xy).map(|c| Move(p.to_string(), c))
    }
}

const MAX_COORD: u16 = 19;

impl KataCoordOrPass {
    pub fn from(maybe_xy: Option<Coord>) -> Result<Self, CoordOutOfRange> {
        if let Some(xy) = maybe_xy {
            if xy.x > MAX_COORD || xy.y > MAX_COORD {
                Err(CoordOutOfRange)
            } else {
                Ok(KataCoordOrPass(format!("({},{})", xy.x, xy.y)))
            }
        } else {
            Ok(KataCoordOrPass(PASS.to_string()))
        }
    }
}

impl KataGoQuery {
    pub fn from(compute_move: ComputeMove) -> Result<Self, CoordOutOfRange> {
        let game_id = compute_move.game_id;
        let game_state = compute_move.game_state;
        let moves_with_errors: Vec<Result<Move, CoordOutOfRange>> = game_state
            .moves
            .iter()
            .map(|gsm| Move::from(gsm.player, gsm.coord))
            .collect();

        if moves_with_errors.iter().any(|m| m.is_err()) {
            Err(CoordOutOfRange)
        } else {
            let moves = moves_with_errors
                .iter()
                .filter_map(|m| m.as_ref().ok())
                .cloned()
                .collect();

            Ok(KataGoQuery {
                id: Id(format!(
                    "{}_{}_{}",
                    game_id.0,
                    game_state.turn,
                    game_state.player_up.to_string()
                )),
                moves,
                board_x_size: game_state.board.size,
                board_y_size: game_state.board.size,
                max_visits: compute_move.max_visits,
                ..Default::default()
            })
        }
    }

    pub fn to_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        Ok(format!("{}\n", &serde_json::to_string(self)?)
            .as_bytes()
            .to_vec())
    }
}

impl KataGoResponse {
    pub fn game_id(&self) -> Result<GameId, KataGoParseErr> {
        let parts: Vec<&str> = self.id.0.split('_').collect();
        if parts.is_empty() {
            Err(KataGoParseErr::WrongFormat)
        } else {
            Ok(GameId(Uuid::from_str(parts[0])?))
        }
    }

    pub fn player(&self) -> Result<Player, KataGoParseErr> {
        let parts: Vec<&str> = self.id.0.split('_').collect();
        if parts.len() < 2 {
            Err(KataGoParseErr::WrongFormat)
        } else {
            Ok(Player::from_str(parts[2]))
        }
    }
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
            max_visits: None,
        }
    }
}

impl Default for Rules {
    fn default() -> Self {
        Rules("tromp-taylor".to_string())
    }
}

impl Default for Komi {
    fn default() -> Self {
        Komi(6.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn basic_move() -> ComputeMove {
        let game_id = GameId(Uuid::nil());
        let game_state = GameState {
            moves: vec![],
            turn: 1,
            player_up: Player::BLACK,
            captures: Captures::default(),
            board: Board::default(),
            game_id: game_id.clone(),
        };
        ComputeMove {
            game_id,
            game_state,
            max_visits: None,
        }
    }

    #[test]
    fn query_from_game_state() {
        let game_id = GameId(Uuid::nil());
        let game_state = GameState {
            moves: vec![
                MoveMade {
                    coord: Some(Coord::of(0, 0)),
                    event_id: EventId::new(),
                    game_id: game_id.clone(),
                    reply_to: ReqId(Uuid::nil()),
                    player: Player::BLACK,
                    captured: vec![],
                },
                MoveMade {
                    coord: Some(Coord::of(1, 1)),
                    event_id: EventId::new(),
                    game_id: game_id.clone(),
                    reply_to: ReqId(Uuid::nil()),
                    player: Player::WHITE,
                    captured: vec![],
                },
                MoveMade {
                    coord: None,
                    event_id: EventId::new(),
                    game_id: game_id.clone(),
                    reply_to: ReqId(Uuid::nil()),
                    player: Player::BLACK,
                    captured: vec![],
                },
            ],
            turn: 4,
            player_up: Player::WHITE,
            captures: Captures::default(),
            board: Board::default(),
            game_id: game_id.clone(),
        };
        let compute_move = ComputeMove {
            game_id,
            game_state,
            max_visits: None,
        };

        let expected = KataGoQuery {
            id: Id("00000000-0000-0000-0000-000000000000_4_WHITE".to_string()),
            moves: vec![
                Move("B".to_string(), KataCoordOrPass("(0,0)".to_string())),
                Move("W".to_string(), KataCoordOrPass("(1,1)".to_string())),
                Move("B".to_string(), KataCoordOrPass("pass".to_string())),
            ],
            ..KataGoQuery::default()
        };

        let actual = KataGoQuery::from(compute_move).expect("move(s) out of range");
        assert_eq!(actual, expected)
    }

    #[test]
    fn oversized_coordinates_rejected() {
        let game_id = GameId(Uuid::nil());
        let game_state = GameState {
            moves: vec![MoveMade {
                coord: Some(Coord::of(999, 999)),
                event_id: EventId::new(),
                game_id: game_id.clone(),
                reply_to: ReqId(Uuid::nil()),
                player: Player::BLACK,
                captured: vec![],
            }],
            turn: 2,
            captures: Captures::default(),
            board: Board::default(),
            player_up: Player::WHITE,
            game_id: game_id.clone(),
        };
        let compute_move = ComputeMove {
            game_id,
            game_state,
            max_visits: None,
        };

        assert!(KataGoQuery::from(compute_move).is_err())
    }

    #[test]
    fn board_size_honored() {
        let game_id = GameId(Uuid::nil());
        let game_state = GameState {
            moves: vec![],
            turn: 1,
            player_up: Player::WHITE,
            board: Board {
                pieces: std::collections::HashMap::new(),
                size: 9,
            },
            game_id: game_id.clone(),
            captures: Captures::default(),
        };
        let compute_move = ComputeMove {
            game_id,
            game_state,
            max_visits: None,
        };

        let expected = KataGoQuery {
            id: Id("00000000-0000-0000-0000-000000000000_1_WHITE".to_string()),
            moves: vec![],
            board_x_size: 9,
            board_y_size: 9,
            ..KataGoQuery::default()
        };

        let actual = KataGoQuery::from(compute_move).expect("move(s) out of range");
        assert_eq!(actual, expected)
    }

    #[test]
    fn max_visits_can_be_specified() {
        let game_id = GameId(Uuid::nil());
        let game_state = GameState {
            moves: vec![],
            turn: 1,
            player_up: Player::BLACK,
            captures: Captures::default(),
            board: Board::default(),
            game_id: game_id.clone(),
        };
        let compute_move = ComputeMove {
            game_id,
            game_state,
            max_visits: Some(25),
        };

        let expected = KataGoQuery {
            id: Id("00000000-0000-0000-0000-000000000000_1_BLACK".to_string()),
            moves: vec![],
            max_visits: Some(25),
            ..KataGoQuery::default()
        };

        let actual = KataGoQuery::from(compute_move).expect("move(s) out of range");
        assert_eq!(actual, expected)
    }

    #[test]
    fn max_visits_skipped_when_none() {
        let compute_move = basic_move();

        let query = KataGoQuery::from(compute_move);
        let json = serde_json::to_string(&query).expect("ser");

        assert!(!json.contains("maxVisits"))
    }

    #[test]
    fn komi_is_always_explicit() {
        let compute_move = basic_move();

        let query = KataGoQuery::from(compute_move).expect("query formed");

        assert_eq!(query.komi, Komi::default())
    }

    #[test]
    fn komi_json_ser() {
        let compute_move = basic_move();

        let json = serde_json::to_string(&KataGoQuery::from(compute_move).expect("query formed"))
            .expect("json");

        assert!(json.contains("\"komi\":6.5"))
    }
}
