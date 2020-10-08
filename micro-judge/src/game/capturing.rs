use move_model::*;
use std::collections::HashSet;

pub fn captures_for(player: Player, placement: Coord, board: &Board) -> HashSet<Coord> {
    let enemy_neighbors: HashSet<(Coord, Player)> = neighbor_pieces(placement, board)
        .iter()
        .filter(|(_, pp)| player != *pp)
        .cloned()
        .collect();
    let mut h = HashSet::new();
    for (target, _) in enemy_neighbors {
        if dead_from(target, placement, board) {
            for i in connected(target, board) {
                h.insert(i);
            }
        }
    }
    h
}

/// Return all open spaces connected to the target piece's formation
fn liberties(target: Coord, board: &Board) -> HashSet<Coord> {
    let mut h = HashSet::new();
    for c in connected(target, &board) {
        for s in neighbor_spaces(c, board) {
            h.insert(s);
        }
    }
    h
}

fn neighbors(target: Coord, board: &Board) -> HashSet<(Coord, Option<Player>)> {
    const OFFSETS: [(i16, i16); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let mut h = HashSet::new();
    let coords = OFFSETS
        .iter()
        .map(|(x, y)| (x + target.x as i16, y + target.y as i16))
        .filter(|(x, y)| {
            !(x < &0 || x >= &(board.size as i16) || y < &0 || y >= &(board.size as i16))
        })
        .map(|(x, y)| Coord {
            x: x as u16,
            y: y as u16,
        });
    for coord in coords {
        h.insert((coord, board.pieces.get(&coord).map(|p| *p)));
    }
    h
}

/// Return neighboring empty spaces
fn neighbor_spaces(target: Coord, board: &Board) -> HashSet<Coord> {
    neighbors(target, board)
        .iter()
        .filter_map(|(coord, player)| {
            if let Some(_) = player {
                None
            } else {
                Some(coord)
            }
        })
        .cloned()
        .collect()
}

/// Return neighbor pieces on all sides of the target
fn neighbor_pieces(target: Coord, board: &Board) -> HashSet<(Coord, Player)> {
    let mut h = HashSet::new();
    for pair in neighbors(target, board)
        .iter()
        .filter_map(|(coord, player)| {
            if let Some(p) = player {
                Some((coord.clone(), p.clone()))
            } else {
                None
            }
        })
    {
        h.insert(pair);
    }
    h
}

fn dead_from(target: Coord, placement: Coord, board: &Board) -> bool {
    liberties(target, board) == [placement].iter().cloned().collect()
}

/// Return all pieces of the same color, connected to the target.  Includes the target itself.
fn connected(target: Coord, board: &Board) -> HashSet<Coord> {
    fn same_color_pieces(player: Player, targs: &HashSet<Coord>, board: &Board) -> Vec<Coord> {
        targs
            .iter()
            .filter_map(|c| {
                let found = board.pieces.get(c);
                found.map(|f| (c, f))
            })
            .filter(|(_, p)| **p == player)
            .map(|(c, _)| *c)
            .collect()
    }
    fn same_color_neighbors(player: Player, ps: &Vec<Coord>, board: &Board) -> HashSet<Coord> {
        let mut r = HashSet::new();
        for coord in ps {
            for (npc, npp) in neighbor_pieces(*coord, board) {
                if npp == player {
                    r.insert(npc);
                }
            }
        }
        r
    }
    if let Some(player) = board.pieces.get(&target) {
        let mut acc: HashSet<Coord> = {
            let mut hh = HashSet::new();
            hh.insert(target);
            hh
        };
        let mut targets: HashSet<Coord> = acc.clone();

        loop {
            let scns =
                same_color_neighbors(*player, &same_color_pieces(*player, &targets, board), board);

            if acc.is_superset(&scns) {
                break;
            } else {
                let next_acc = acc.union(&scns);
                targets = scns.clone();
                acc = next_acc.cloned().collect();
            }
        }
        acc
    } else {
        HashSet::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn correct_neighbor_pieces() {
        let pieces: HashMap<Coord, Player> = [
            (Coord { x: 0, y: 0 }, Player::BLACK),
            (Coord { x: 1, y: 0 }, Player::BLACK),
            (Coord { x: 2, y: 0 }, Player::BLACK),
            (Coord { x: 0, y: 1 }, Player::WHITE),
            (Coord { x: 1, y: 1 }, Player::WHITE),
            (Coord { x: 2, y: 1 }, Player::BLACK),
            (Coord { x: 0, y: 2 }, Player::WHITE),
            (Coord { x: 1, y: 2 }, Player::WHITE),
            (Coord { x: 2, y: 2 }, Player::WHITE),
            (Coord { x: 4, y: 3 }, Player::BLACK),
            (Coord { x: 1, y: 5 }, Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();

        let board = Board {
            pieces,
            ..Default::default()
        };

        let actual = neighbor_pieces(Coord { x: 1, y: 1 }, &board);

        let expected: HashSet<(Coord, Player)> = [
            (Coord { x: 1, y: 0 }, Player::BLACK),
            (Coord { x: 0, y: 1 }, Player::WHITE),
            (Coord { x: 2, y: 1 }, Player::BLACK),
            (Coord { x: 1, y: 2 }, Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();

        assert_eq!(expected, actual)
    }

    #[test]
    fn edge_neighbor_pieces() {
        let pieces: HashMap<Coord, Player> = [
            (Coord { x: 0, y: 0 }, Player::BLACK),
            (Coord { x: 1, y: 0 }, Player::BLACK),
            (Coord { x: 2, y: 0 }, Player::BLACK),
            (Coord { x: 0, y: 1 }, Player::WHITE),
            (Coord { x: 1, y: 1 }, Player::WHITE),
            (Coord { x: 2, y: 1 }, Player::BLACK),
            (Coord { x: 0, y: 2 }, Player::WHITE),
            (Coord { x: 1, y: 2 }, Player::WHITE),
            (Coord { x: 2, y: 2 }, Player::WHITE),
            (Coord { x: 4, y: 3 }, Player::BLACK),
            (Coord { x: 1, y: 5 }, Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();

        let board = Board {
            pieces,
            ..Default::default()
        };

        let actual = neighbor_pieces(Coord { x: 0, y: 1 }, &board);

        let expected: HashSet<(Coord, Player)> = [
            (Coord { x: 0, y: 0 }, Player::BLACK),
            (Coord { x: 1, y: 1 }, Player::WHITE),
            (Coord { x: 0, y: 2 }, Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(expected, actual)
    }

    #[test]
    fn no_empty_neighbor_pieces() {
        let pieces: HashMap<Coord, Player> = [
            (Coord { x: 0, y: 0 }, Player::BLACK),
            (Coord { x: 1, y: 0 }, Player::BLACK),
            (Coord { x: 2, y: 0 }, Player::BLACK),
            (Coord { x: 0, y: 1 }, Player::WHITE),
            (Coord { x: 1, y: 1 }, Player::WHITE),
            (Coord { x: 2, y: 1 }, Player::BLACK),
            (Coord { x: 0, y: 2 }, Player::WHITE),
            (Coord { x: 1, y: 2 }, Player::WHITE),
            (Coord { x: 2, y: 2 }, Player::WHITE),
            (Coord { x: 4, y: 3 }, Player::BLACK),
            (Coord { x: 1, y: 5 }, Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();
        let board = Board {
            pieces,
            ..Default::default()
        };
        let actual = neighbor_pieces(Coord::of(4, 4), &board);
        let mut expected = HashSet::new();
        expected.insert((Coord::of(4, 3), Player::BLACK));
        assert_eq!(expected, actual)
    }

    #[test]
    fn connected_test() {
        let pieces: HashMap<Coord, Player> = [
            (Coord::of(0, 0), Player::BLACK),
            (Coord::of(1, 0), Player::BLACK),
            (Coord::of(2, 0), Player::BLACK),
            (Coord::of(0, 1), Player::WHITE),
            (Coord::of(1, 1), Player::WHITE),
            (Coord::of(2, 1), Player::BLACK),
            (Coord::of(0, 2), Player::WHITE),
            (Coord::of(1, 2), Player::WHITE),
            (Coord::of(2, 2), Player::WHITE),
            (Coord::of(4, 3), Player::BLACK),
            (Coord::of(1, 3), Player::WHITE),
            (Coord::of(1, 4), Player::WHITE),
            (Coord::of(1, 5), Player::WHITE),
            (Coord::of(5, 1), Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();
        let board = Board {
            pieces: pieces.clone(),
            ..Default::default()
        };
        let actual = connected(Coord::of(1, 4), &board);
        let mut expected: HashSet<Coord> = HashSet::new();
        for (c, p) in pieces {
            if p == Player::WHITE && (c, p) != (Coord::of(5, 1), Player::WHITE) {
                expected.insert(c);
            }
        }
        assert_eq!(expected, actual)
    }

    #[test]
    fn so_connected_test() {
        let pieces: HashMap<Coord, Player> = [
            (Coord::of(0, 0), Player::BLACK),
            (Coord::of(1, 0), Player::BLACK),
            (Coord::of(2, 0), Player::BLACK),
            (Coord::of(0, 1), Player::WHITE),
            (Coord::of(1, 1), Player::WHITE),
            (Coord::of(2, 1), Player::BLACK),
            (Coord::of(0, 2), Player::WHITE),
            (Coord::of(1, 2), Player::WHITE),
            (Coord::of(2, 2), Player::WHITE),
            (Coord::of(4, 3), Player::BLACK),
            (Coord::of(1, 3), Player::WHITE),
            (Coord::of(1, 4), Player::WHITE),
            (Coord::of(1, 5), Player::WHITE),
            (Coord::of(5, 1), Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();

        let board = Board {
            pieces: pieces.clone(),
            ..Default::default()
        };
        let actual = connected(Coord { x: 1, y: 0 }, &board);
        let e1: Vec<(Coord, Player)> = pieces
            .iter()
            .filter(|(_, v)| **v == Player::BLACK)
            .map(|(k, v)| (*k, *v))
            .collect();
        let expected: HashSet<Coord> = e1
            .iter()
            .filter(|(k, v)| (k, v) != (&Coord { x: 4, y: 3 }, &Player::BLACK))
            .map(|(k, _)| k)
            .cloned()
            .collect();
        assert_eq!(expected, actual)
    }

    #[test]
    fn connections_empty() {
        let pieces: HashMap<Coord, Player> = [
            (Coord::of(0, 0), Player::BLACK),
            (Coord::of(1, 0), Player::BLACK),
            (Coord::of(2, 0), Player::BLACK),
        ]
        .iter()
        .cloned()
        .collect();
        let board = Board {
            pieces,
            ..Default::default()
        };
        let actual = connected(Coord { x: 4, y: 4 }, &board);
        let expected: HashSet<Coord> = HashSet::new();
        assert_eq!(expected, actual)
    }

    #[test]
    fn basic_liberties() {
        let pieces: HashMap<Coord, Player> = [
            (Coord::of(0, 0), Player::BLACK),
            (Coord::of(1, 0), Player::BLACK),
            (Coord::of(2, 0), Player::BLACK),
            (Coord::of(0, 1), Player::WHITE),
            (Coord::of(1, 1), Player::WHITE),
            (Coord::of(2, 1), Player::BLACK),
            (Coord::of(0, 2), Player::WHITE),
            (Coord::of(1, 2), Player::WHITE),
            (Coord::of(2, 2), Player::WHITE),
            (Coord::of(4, 3), Player::BLACK),
            (Coord::of(1, 3), Player::WHITE),
            (Coord::of(1, 4), Player::WHITE),
            (Coord::of(1, 5), Player::WHITE),
            (Coord::of(5, 1), Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();

        let board = Board {
            pieces,
            ..Default::default()
        };

        let actual = liberties(Coord { x: 1, y: 3 }, &board);
        let expected: HashSet<Coord> = [
            Coord::of(3, 2),
            Coord::of(0, 3),
            Coord::of(2, 3),
            Coord::of(0, 4),
            Coord::of(2, 4),
            Coord::of(0, 5),
            Coord::of(2, 5),
            Coord::of(1, 6),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(expected, actual)
    }

    #[test]
    fn more_freedoms() {
        let pieces: HashMap<Coord, Player> = [
            (Coord::of(0, 0), Player::BLACK),
            (Coord::of(1, 0), Player::BLACK),
            (Coord::of(2, 0), Player::BLACK),
            (Coord::of(0, 1), Player::WHITE),
            (Coord::of(1, 1), Player::WHITE),
            (Coord::of(2, 1), Player::BLACK),
            (Coord::of(0, 2), Player::WHITE),
            (Coord::of(1, 2), Player::WHITE),
            (Coord::of(2, 2), Player::WHITE),
            (Coord::of(4, 3), Player::BLACK),
            (Coord::of(1, 3), Player::WHITE),
            (Coord::of(1, 4), Player::WHITE),
            (Coord::of(1, 5), Player::WHITE),
            (Coord::of(5, 1), Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();
        let board = Board {
            pieces,
            ..Default::default()
        };
        let actual = liberties(Coord { x: 0, y: 0 }, &board);
        let expected: HashSet<Coord> = [Coord { x: 3, y: 0 }, Coord { x: 3, y: 1 }]
            .iter()
            .cloned()
            .collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn take_over_the_world() {
        let pieces: HashMap<Coord, Player> = [
            (Coord::of(0, 0), Player::BLACK),
            (Coord::of(1, 0), Player::BLACK),
            (Coord::of(2, 0), Player::BLACK),
            (Coord::of(3, 0), Player::WHITE),
            (Coord::of(0, 1), Player::WHITE),
            (Coord::of(1, 1), Player::WHITE),
            (Coord::of(2, 1), Player::BLACK),
            (Coord::of(0, 2), Player::WHITE),
            (Coord::of(1, 2), Player::WHITE),
            (Coord::of(2, 2), Player::WHITE),
            (Coord::of(4, 3), Player::BLACK),
            (Coord::of(1, 5), Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();
        let board = Board {
            pieces,
            ..Default::default()
        };
        let actual = captures_for(Player::WHITE, Coord { x: 3, y: 1 }, &board);

        let expected: HashSet<Coord> = [
            Coord::of(0, 0),
            Coord::of(1, 0),
            Coord::of(2, 0),
            Coord::of(2, 1),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(expected, actual)
    }

    #[test]
    fn not_too_greedy() {
        let pieces: HashMap<Coord, Player> = [
            (Coord::of(0, 0), Player::BLACK),
            (Coord::of(1, 0), Player::BLACK),
            (Coord::of(2, 0), Player::BLACK),
            (Coord::of(0, 1), Player::WHITE),
            (Coord::of(1, 1), Player::WHITE),
            (Coord::of(2, 1), Player::BLACK),
            (Coord::of(0, 2), Player::WHITE),
            (Coord::of(1, 2), Player::WHITE),
            (Coord::of(2, 2), Player::WHITE),
            (Coord::of(4, 3), Player::BLACK),
            (Coord::of(1, 5), Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();
        let board = Board {
            pieces,
            ..Default::default()
        };
        let actual = captures_for(Player::WHITE, Coord::of(3, 1), &board);
        let expected: HashSet<Coord> = HashSet::new();
        assert_eq!(expected, actual);
    }

    #[test]
    fn capture_other_side() {
        let pieces: HashMap<Coord, Player> = [
            (Coord::of(0, 0), Player::BLACK),
            (Coord::of(1, 0), Player::BLACK),
            (Coord::of(2, 0), Player::BLACK),
            (Coord::of(0, 1), Player::WHITE),
            (Coord::of(1, 1), Player::WHITE),
            (Coord::of(2, 1), Player::BLACK),
            (Coord::of(0, 2), Player::WHITE),
            (Coord::of(1, 2), Player::WHITE),
            (Coord::of(1, 3), Player::WHITE),
            (Coord::of(1, 4), Player::WHITE),
            (Coord::of(1, 5), Player::WHITE),
            (Coord::of(2, 2), Player::WHITE),
            (Coord::of(4, 3), Player::BLACK),
            (Coord::of(3, 2), Player::BLACK),
            (Coord::of(0, 3), Player::BLACK),
            (Coord::of(2, 3), Player::BLACK),
            (Coord::of(0, 4), Player::BLACK),
            (Coord::of(2, 4), Player::BLACK),
            (Coord::of(0, 5), Player::BLACK),
            (Coord::of(2, 5), Player::BLACK),
        ]
        .iter()
        .cloned()
        .collect();
        let board = Board {
            pieces: pieces.clone(),
            ..Default::default()
        };
        let actual = captures_for(Player::BLACK, Coord::of(1, 6), &board);
        let mut expected = HashSet::new();
        for (coo, pla) in pieces {
            if !(pla == Player::BLACK || coo == Coord::of(5, 1)) {
                expected.insert(coo);
            }
        }
        assert_eq!(expected, actual)
    }

    #[test]
    fn corner_capture_full_size_board() {
        let pieces: HashMap<Coord, Player> = [
            (Coord::of(18, 18), Player::BLACK),
            (Coord::of(18, 17), Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();
        let board = Board {
            pieces,
            ..Default::default()
        };
        let actual = captures_for(Player::WHITE, Coord::of(17, 18), &board);
        let mut expected = HashSet::new();
        expected.insert(Coord::of(18, 18));
        assert_eq!(expected, actual)
    }

    #[test]
    fn corner_liberty() {
        let pieces: HashMap<Coord, Player> = [
            (Coord::of(18, 18), Player::BLACK),
            (Coord::of(18, 17), Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();
        let board = Board {
            pieces,
            ..Default::default()
        };
        let actual = liberties(Coord::of(18, 18), &board);
        let mut expected = HashSet::new();
        expected.insert(Coord::of(17, 18));
        assert_eq!(expected, actual)
    }

    #[test]
    fn connected_includes_self() {
        let pieces: HashMap<Coord, Player> = [
            (Coord::of(18, 18), Player::BLACK),
            (Coord::of(18, 17), Player::WHITE),
        ]
        .iter()
        .cloned()
        .collect();
        let board = Board {
            pieces,
            ..Default::default()
        };
        let actual = connected(Coord::of(18, 18), &board);
        let mut expected = HashSet::new();
        expected.insert(Coord::of(18, 18));
        assert_eq!(expected, actual)
    }
}
