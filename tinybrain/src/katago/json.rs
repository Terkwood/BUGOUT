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
pub struct Move(pub String, pub AlphaNumOrPass);

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
/// Alphanumeric coordinate as expected by KataGo, e.g. `Q16` or `pass`.
/// This isn't strictly necessary, as KataGo supports numeric coords.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
pub struct AlphaNumOrPass(pub String);

impl Move {
    pub fn from(player: Player, maybe_xy: Option<Coord>) -> Result<Self, CoordOutOfRange> {
        let p = match player {
            Player::BLACK => "B",
            _ => "W",
        };
        AlphaNumOrPass::from(maybe_xy).map(|c| Move(p.to_string(), c))
    }
}

impl AlphaNumOrPass {
    pub fn from(maybe_xy: Option<Coord>) -> Result<Self, CoordOutOfRange> {
        if let Some(xy) = maybe_xy {
            let alphabet = (b'A'..=b'Z')
                .filter_map(|c| {
                    let c = c as char;
                    if c.is_alphabetic() {
                        Some(c)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let i = xy.x as usize;
            if i < alphabet.len() {
                Ok(AlphaNumOrPass(format!("{}{}", alphabet[i], xy.y + 1)))
            } else {
                Err(CoordOutOfRange)
            }
        } else {
            Ok(AlphaNumOrPass(PASS.to_string()))
        }
    }
}

pub fn interpret_coord(move_info_move: &str) -> Result<Option<Coord>, CoordOutOfRange> {
    let t = move_info_move.trim();
    if t.to_ascii_lowercase() == PASS {
        Ok(None)
    } else {
        Ok(Some(from_alphanum(&t.to_ascii_uppercase())?))
    }
}

fn from_alphanum(a: &str) -> Result<Coord, CoordOutOfRange> {
    if a.len() < 2 {
        Err(CoordOutOfRange)
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
            Err(CoordOutOfRange)
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
        let parts: Vec<&str> = self.id.0.split("_").collect();
        if parts.is_empty() {
            Err(KataGoParseErr::WrongFormat)
        } else {
            Ok(GameId(Uuid::from_str(parts[0])?))
        }
    }

    pub fn player(&self) -> Result<Player, KataGoParseErr> {
        let parts: Vec<&str> = self.id.0.split("_").collect();
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
    use micro_model_bot::MoveComputed;
    use std::convert::TryFrom;
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
                Move("B".to_string(), AlphaNumOrPass("A1".to_string())),
                Move("W".to_string(), AlphaNumOrPass("B2".to_string())),
                Move("B".to_string(), AlphaNumOrPass("pass".to_string())),
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

    #[test]
    fn test_interpret_coord() {
        assert_eq!(
            interpret_coord("B3").expect("parse"),
            Some(Coord { x: 1, y: 2 })
        );
        assert_eq!(
            interpret_coord("c4").expect("parse"),
            Some(Coord { x: 2, y: 3 })
        );
        assert_eq!(
            interpret_coord(" D5 ").expect("parse"),
            Some(Coord { x: 3, y: 4 })
        )
    }

    #[test]
    fn test_interpret_pass() {
        assert_eq!(interpret_coord("pass").expect("parse"), None);
        assert_eq!(interpret_coord("PASS").expect("parse"), None);
        assert_eq!(interpret_coord(" PaSs   ").expect("parse"), None)
    }

    #[test]
    fn move_computed_from() {
        let actual = MoveComputed::try_from(KataGoResponse {
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
