/// Putting this hack in place until we get motivated
/// enough to deprecate micro_model_moves & micro_model_bot
pub trait Convert<A> {
    fn convert(&self) -> A;
}

impl Convert<micro_model_moves::GameState> for move_model::GameState {
    fn convert(&self) -> micro_model_moves::GameState {
        micro_model_moves::GameState {
            player_up: self.player_up.convert(),
            board: micro_model_moves::Board {
                pieces: todo!(),
                size: self.board.size,
            },
            captures: todo!(),
            moves: todo!(),
            turn: self.turn,
        }
    }
}
impl Convert<micro_model_moves::Player> for move_model::Player {
    fn convert(&self) -> micro_model_moves::Player {
        match self {
            move_model::Player::BLACK => micro_model_moves::Player::BLACK,
            _ => micro_model_moves::Player::WHITE,
        }
    }
}
