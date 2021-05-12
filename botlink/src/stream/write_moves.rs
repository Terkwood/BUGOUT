use super::xadd::XAdder;
use crate::repo::BoardSizeRepo;
use bot_model::{api::MoveComputed, AlphaNumCoord};
use core_model::ReqId;
use crossbeam_channel::{select, Receiver};
use log::error;
use move_model::{Coord, MakeMove};
use std::sync::Arc;
use uuid::Uuid;

pub fn xadd_loop(
    move_computed_out: Receiver<MoveComputed>,
    xadder: Arc<dyn XAdder>,
    board_size_repo: Arc<dyn BoardSizeRepo>,
) {
    loop {
        select! { recv(move_computed_out) -> msg =>
            match msg {
                Ok(MoveComputed { game_id, player, alphanum_coord }) => {
                    if let Ok(board_size) = board_size_repo.get(&game_id) {
                        let coord = alphanum_coord.map(|a|convert(a, board_size));

                        let command = MakeMove  { game_id, player, req_id: ReqId(Uuid::new_v4()), coord };

                        if let Err(e) = xadder.xadd_make_move_command(&command) {
                            error!("could not xadd move command : {:?}",e)
                        }
                    } else {
                        error!("Could not fetch board size for {}", game_id.0)
                    }
                }
                Err(e) =>
                    error!("loop recv: {}", e)
            }
        }
    }
}

fn convert(a: AlphaNumCoord, board_size: u16) -> Coord {
    let r: Vec<char> = (b'A'..=b'Z')
        .filter(|l| l != &b'I')
        .map(char::from)
        .collect();
    let x = r.iter().position(|l| l == &a.0).expect("convert") as u16;

    Coord {
        x,
        y: board_size - a.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_convert() {
        let a = AlphaNumCoord('A', 1);
        let board_size = 9;
        let actual = convert(a, board_size);
        let expected = Coord { x: 0, y: 8 };
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_skip_i() {
        let j = AlphaNumCoord('J', 19);
        let board_size = 19;
        let actual = convert(j, board_size);
        let expected = Coord { x: 8, y: 0 };
        assert_eq!(actual, expected)
    }
}
