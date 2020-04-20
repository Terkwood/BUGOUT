use crate::err::*;

use micro_model_moves::*;
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
    pub fn from(game_id: &GameId, game_state: &GameState) -> Result<Self, CoordOutOfRange> {
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
                ..KataGoQuery::default()
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
        Komi(7.5)
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
            turn: 3,
            player_up: Player::WHITE,
            ..GameState::default()
        };

        let expected = KataGoQuery {
            id: Id("00000000-0000-0000-0000-000000000000_3_WHITE".to_string()),
            moves: vec![
                Move("B".to_string(), KataCoordOrPass("(0,0)".to_string())),
                Move("W".to_string(), KataCoordOrPass("(1,1)".to_string())),
                Move("B".to_string(), KataCoordOrPass("pass".to_string())),
            ],
            ..KataGoQuery::default()
        };

        let actual = KataGoQuery::from(&game_id, &game_state).expect("move(s) out of range");
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
            ..GameState::default()
        };

        assert!(KataGoQuery::from(&game_id, &game_state).is_err())
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
            ..GameState::default()
        };

        let expected = KataGoQuery {
            id: Id("00000000-0000-0000-0000-000000000000_1_WHITE".to_string()),
            moves: vec![],
            board_x_size: 9,
            board_y_size: 9,
            ..KataGoQuery::default()
        };

        let actual = KataGoQuery::from(&game_id, &game_state).expect("move(s) out of range");
        assert_eq!(actual, expected)
    }
}
