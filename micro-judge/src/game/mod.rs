use crate::model::*;

mod capturing;

pub fn validate_move(make_move: MakeMoveCommand, game_state: GameState) -> bool {
    let correct_player = make_move.player == game_state.player_up;
    let coord = make_move.coord;
    let passing = coord.is_none();
    let coord_exists = || {
        if let Some(c) = coord {
            let size = game_state.board.size;
            size > c.x && size > c.y
        } else {
            false
        }
    };
    let valid_coord = || {
        coord_exists()
            && game_state
                .board
                .pieces
                .get(&make_move.coord.unwrap())
                .is_none()
    };
    correct_player && (passing || valid_coord())
}
