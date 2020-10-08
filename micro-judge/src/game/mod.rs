pub use capturing::captures_for;
use core_model::EventId;
use move_model::*;

use log::info;

mod capturing;

pub enum Judgement {
    Accepted(MoveMade),
    Rejected,
}
pub fn judge(mm: &MakeMove, game_state: &GameState) -> Judgement {
    info!("Judge {:?}", mm);
    if validate_move(mm, game_state) {
        let captured: Vec<Coord> = mm
            .coord
            .map(|c| {
                captures_for(mm.player, c, &game_state.board)
                    .iter()
                    .cloned()
                    .collect()
            })
            .unwrap_or(vec![]);

        let move_made = MoveMade {
            player: mm.player,
            coord: mm.coord,
            captured,
            event_id: EventId::new(),
            game_id: mm.game_id.clone(),
            reply_to: mm.req_id.clone(),
        };
        Judgement::Accepted(move_made)
    } else {
        Judgement::Rejected
    }
}

fn validate_move(make_move: &MakeMove, game_state: &GameState) -> bool {
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
