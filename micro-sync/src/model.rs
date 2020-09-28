use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct GameId(pub Uuid);
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionId(pub Uuid);
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ReqId(pub Uuid);
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EventId(pub Uuid);

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
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Move {
    pub player: Player,
    pub coord: Option<Coord>,
    pub turn: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameState {
    pub moves: Option<Vec<MoveEvent>>,
    pub player_up: Player,
}

#[derive(Clone, Deserialize, Debug)]
pub struct MoveMade {
    pub game_id: GameId,
    pub reply_to: ReqId,
    pub event_id: EventId,
    pub player: Player,
    pub coord: Option<Coord>,
    pub captured: Vec<Coord>,
}

impl GameState {
    pub fn to_history(&self) -> Vec<Move> {
        self.moves
            .clone()
            .map(|the_moves| {
                the_moves
                    .iter()
                    .enumerate()
                    .map(|(i, &MoveEvent { player, coord })| Move {
                        turn: (i + 1) as u32,
                        player,
                        coord,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl EventId {
    pub fn new() -> Self {
        EventId(Uuid::new_v4())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn game_state_to_history() {
        let c1 = None;
        let c2 = Some(Coord { x: 0, y: 0 });
        let gs_moves = vec![
            MoveEvent {
                coord: c1,
                player: Player::BLACK,
            },
            MoveEvent {
                coord: c2,
                player: Player::WHITE,
            },
        ];
        let player_up = Player::BLACK;
        let game_state = GameState {
            moves: Some(gs_moves),
            player_up,
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
