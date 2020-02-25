use crate::model::*;
use std::collections::HashSet;

pub fn captures_for(player: Player, placement: Coord, board: Board) -> HashSet<Coord> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn correct_neighbor_pieces() {
        todo!()
    }

    #[test]
    fn edge_neighbor_pieces() {
        todo!()
    }

    #[test]
    fn no_empty_neighbor_pieces() {
        todo!()
    }

    #[test]
    fn connected_test() {
        todo!()
    }

    #[test]
    fn so_connected_test() {
        todo!()
    }

    #[test]
    fn connections_empty() {
        todo!()
    }

    #[test]
    fn basic_liberties() {
        todo!()
    }

    #[test]
    fn more_freedoms() {
        todo!()
    }

    #[test]
    fn take_over_the_world() {
        todo!()
    }

    #[test]
    fn not_too_greedy() {
        todo!()
    }

    #[test]
    fn capture_other_side() {
        todo!()
    }

    #[test]
    fn corner_capture_full_size_board() {
        todo!()
    }

    #[test]
    fn corner_liberty() {
        todo!()
    }

    #[test]
    fn connected_includes_self() {todo!()}
}
