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
pub struct Move(pub String, pub AlphaNumCoord);

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd)]
pub struct Rules(pub String);

#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub struct Komi(pub f32);

#[derive(Debug, Clone, Deserialize, PartialEq, PartialOrd)]
pub struct MoveInfo {
    pub order: u32,
    pub r#move: String,
}

/// Alphanumeric coordinate as expected by KataGo, e.g. `Q16`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
pub struct AlphaNumCoord(pub String);

impl Move {
    pub fn from(player: Player, xy: Coord) -> Result<Self, CoordOutOfRange> {
        let p = match player {
            Player::BLACK => "B",
            _ => "W",
        };
        AlphaNumCoord::from(xy).map(|c| Move(p.to_string(), c))
    }
}

impl AlphaNumCoord {
    pub fn from(xy: Coord) -> Result<Self, CoordOutOfRange> {
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
            Ok(AlphaNumCoord(format!("{}{}", alphabet[i], xy.y + 1)))
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
            .map(|gsm| Move::from(gsm.player, gsm.coord.unwrap())) // TODO pass
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
                    coord: Some(Coord::of(10, 10)),
                    event_id: EventId::new(),
                    game_id: game_id.clone(),
                    reply_to: ReqId(Uuid::nil()),
                    player: Player::BLACK,
                    captured: vec![],
                },
            ],
            turn: 3, // TODO be careful of differing turn index
            player_up: Player::WHITE,
            ..GameState::default()
        };

        let expected = KataGoQuery {
            id: Id("00000000-0000-0000-0000-000000000000_3_WHITE".to_string()),
            moves: vec![
                Move("B".to_string(), AlphaNumCoord("A1".to_string())),
                Move("W".to_string(), AlphaNumCoord("B2".to_string())),
                Move("B".to_string(), AlphaNumCoord("K11".to_string())),
            ],
            ..KataGoQuery::default()
        };

        let actual = KataGoQuery::from(&game_id, &game_state).expect("move(s) out of range");
        assert_eq!(actual, expected);
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
}
