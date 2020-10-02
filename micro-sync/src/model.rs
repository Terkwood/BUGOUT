use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

pub use core_model::*;
pub use move_model::*;
pub use sync_model::*;
/*
#[derive(Clone, Serialize, Deserialize, Debug, Copy, PartialEq)]
pub enum Player {
    BLACK,
    WHITE,
}

#[derive(Clone, Serialize, Deserialize, Debug, Copy, PartialEq)]
pub struct Coord {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MoveEvent {
    pub player: Player,
    pub coord: Option<Coord>,
}*/

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Move {
    pub player: Player,
    pub coord: Option<Coord>,
    pub turn: u32,
}
/*
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameState {
    pub moves: Option<Vec<MoveEvent>>,
    pub player_up: Player,
}
*/

pub trait ToHistory {
    fn to_history(&self) -> Vec<Move>;
}

impl ToHistory for GameState {
    fn to_history(&self) -> Vec<Move> {
        self.moves
            .iter()
            .enumerate()
            .map(|(i, mm)| Move {
                turn: (i + 1) as u32,
                player: mm.player,
                coord: mm.coord,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn game_state_to_history() {
        let c1 = None;
        let c2 = Some(Coord { x: 0, y: 0 });
        let fake_game_id = GameId::new();
        let gs_moves = vec![
            MoveMade {
                coord: c1,
                player: Player::BLACK,
                reply_to: ReqId::new(),
                captured: Vec::new(),
                event_id: EventId::new(),
                game_id: fake_game_id.clone(),
            },
            MoveMade {
                coord: c2,
                player: Player::WHITE,
                captured: Vec::new(),
                event_id: EventId::new(),
                reply_to: ReqId::new(),
                game_id: fake_game_id,
            },
        ];
        let player_up = Player::BLACK;
        let game_state = GameState {
            moves: gs_moves,
            player_up,
            ..Default::default()
        };

        let actual = game_state.to_history();
        let expected: Vec<Move> = vec![
            Move {
                coord: c1,
                turn: 1,
                player: Player::BLACK,
            },
            Move {
                coord: c2,
                player: Player::WHITE,
                turn: 2,
            },
        ];
        assert_eq!(actual, expected)
    }
}
