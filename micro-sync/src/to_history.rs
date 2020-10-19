use move_model::*;
use sync_model::*;

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
    use core_model::*;
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
                game_id: fake_game_id.clone(),
            },
        ];
        let player_up = Player::BLACK;
        let game_state = GameState {
            moves: gs_moves,
            player_up,
            captures: Captures::default(),
            game_id: fake_game_id,
            board: Board::default(),
            turn: 1,
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
