use super::*;
use crate::repo::Botness;
use crate::Components;
use move_model::{Board, Captures, MoveMade, Player};

pub fn consume_undo(um: &UndoMove, reg: &Components) -> Result<(), UndoProcessingErr> {
    let botness = reg.botness_repo.get(&um.game_id, um.player)?;
    let requester_is_human = botness == Botness::IsHuman;

    if let Some(game_state) = reg.game_state_repo.get(&um.game_id)? {
        let player_up_is_human: bool = requester_is_human && game_state.player_up == um.player;
        let at_least_two_moves_made: bool = game_state.moves.len() > 1;

        if player_up_is_human && at_least_two_moves_made {
            let rolled_back = rollback(&game_state);
            reg.xadd.xadd(&StreamOutput::LOG(rolled_back.clone()))?;
            reg.xadd.xadd(&StreamOutput::MU(MoveUndone {
                game_id: um.game_id.clone(),
                player: um.player,
                game_state: rolled_back,
            }))?;
        } else {
            reject(um, reg)?
        }
    } else {
        reject(um, reg)?
    }

    Ok(())
}

fn reject(undo_move: &UndoMove, reg: &Components) -> Result<(), StreamAddErr> {
    reg.xadd.xadd(&StreamOutput::REJECT(undo_move.clone()))
}

fn rollback(game_state: &GameState) -> GameState {
    let moves = game_state.moves[0..(&game_state.moves.len() - 2)].to_vec();

    GameState {
        game_id: game_state.game_id.clone(),
        turn: game_state.turn - 2,
        player_up: game_state.player_up,
        moves,
        board: compute_board(&game_state.moves, game_state.board.size),
        captures: compute_captures(&game_state.moves),
    }
}

fn compute_captures(moves: &[MoveMade]) -> Captures {
    let mut out = Captures::default();
    for m in moves.iter() {
        let c = m.captured.len() as u16;
        if m.player == Player::BLACK {
            out.black += c
        } else {
            out.white += c
        }
    }
    out
}
fn compute_board(moves: &[MoveMade], size: u16) -> Board {
    let mut out = Board {
        size,
        ..Default::default()
    };
    for m in moves.iter() {
        if let Some(coord) = m.coord {
            out.pieces.insert(coord, m.player);
        }
    }
    out
}

use crate::repo::RepoErr;

#[derive(Debug)]
pub enum UndoProcessingErr {
    Repo(RepoErr),
    StreamAdd(StreamAddErr),
}
impl From<crate::repo::RepoErr> for UndoProcessingErr {
    fn from(e: RepoErr) -> Self {
        Self::Repo(e)
    }
}
impl From<StreamAddErr> for UndoProcessingErr {
    fn from(e: StreamAddErr) -> Self {
        Self::StreamAdd(e)
    }
}
