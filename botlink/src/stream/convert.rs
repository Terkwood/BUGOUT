/// Putting this hack in place until we get motivated
/// enough to deprecate micro_model_moves & micro_model_bot
pub trait Convert<A> {
    fn convert(&self) -> A;
}

impl Convert<micro_model_moves::GameState> for move_model::GameState {
    fn convert(&self) -> micro_model_moves::GameState {
        micro_model_moves::GameState {
            player_up: self.player_up.convert(),
            captures: self.captures.convert(),
            board: self.board.convert(),
            moves: self.moves.iter().map(|m| m.convert()).collect(),
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
impl Convert<micro_model_moves::Board> for move_model::Board {
    fn convert(&self) -> micro_model_moves::Board {
        micro_model_moves::Board {
            pieces: self
                .pieces
                .iter()
                .map(|(k, v)| (k.convert(), v.convert()))
                .collect(),
            size: self.size,
        }
    }
}
impl Convert<micro_model_moves::Captures> for move_model::Captures {
    fn convert(&self) -> micro_model_moves::Captures {
        micro_model_moves::Captures {
            black: self.black,
            white: self.white,
        }
    }
}
impl Convert<micro_model_moves::Coord> for move_model::Coord {
    fn convert(&self) -> micro_model_moves::Coord {
        micro_model_moves::Coord {
            x: self.x,
            y: self.y,
        }
    }
}
impl Convert<micro_model_moves::MoveMade> for move_model::MoveMade {
    fn convert(&self) -> micro_model_moves::MoveMade {
        micro_model_moves::MoveMade {
            game_id: self.game_id.convert(),
            captured: self.captured.iter().map(|c| c.convert()).collect(),
            coord: self.coord.map(|c| c.convert()),
            event_id: self.event_id.convert(),
            player: self.player.convert(),
            reply_to: self.reply_to.convert(),
        }
    }
}
impl Convert<micro_model_moves::GameId> for core_model::GameId {
    fn convert(&self) -> micro_model_moves::GameId {
        micro_model_moves::GameId(self.0)
    }
}

impl Convert<micro_model_moves::EventId> for core_model::EventId {
    fn convert(&self) -> micro_model_moves::EventId {
        micro_model_moves::EventId(self.0)
    }
}

impl Convert<micro_model_moves::ReqId> for core_model::ReqId {
    fn convert(&self) -> micro_model_moves::ReqId {
        micro_model_moves::ReqId(self.0)
    }
}
