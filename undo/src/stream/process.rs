use super::*;
use crate::repo::Botness;
use crate::Components;
use log::{error, info};
use move_model::{Board, Captures, MoveMade};
use redis_streams::XReadEntryId;

pub fn process(reg: &Components) {
    let mut unacked = Unacknowledged::default();
    loop {
        match reg.xread.xread_sorted() {
            Ok(xrr) => {
                for (xid, data) in xrr {
                    info!("🧮 Processing {:?}", &data);
                    consume(xid, &data, &reg);
                    unacked.push(xid, data);
                }
            }
            Err(e) => error!("Stream err {:?}", e),
        }

        unacked.ack_all(&reg)
    }
}

fn consume(_xid: XReadEntryId, event: &StreamInput, reg: &Components) {
    match event {
        StreamInput::LOG(game_state) => consume_log(game_state, reg),
        StreamInput::BA(bot_attached) => consume_bot_attached(bot_attached, reg),
        StreamInput::UM(undo_move) => {
            if let Err(e) = consume_undo(undo_move, reg) {
                error!("could not process undo move event {:?}", e)
            }
        }
    }
}

fn consume_log(game_state: &GameState, reg: &Components) {
    if let Err(e) = reg.game_state_repo.put(&game_state) {
        error!("could not track game state: {:?}", e)
    }
}

fn consume_bot_attached(ba: &BotAttached, reg: &Components) {
    if let Err(e) = reg.botness_repo.put(&ba.game_id, ba.player, Botness::IsBot) {
        error!("could not track bot attached: {:?}", e)
    }
}

fn consume_undo(um: &UndoMove, reg: &Components) -> Result<(), UndoProcessingErr> {
    let botness = reg.botness_repo.get(&um.game_id, um.player)?;
    let requester_is_human = botness == Botness::IsHuman;

    if let Some(game_state) = reg.game_state_repo.get(&um.game_id)? {
        let player_up_is_human: bool = requester_is_human && game_state.player_up == um.player;
        let at_least_two_moves_made: bool = game_state.moves.len() > 1;

        if player_up_is_human && at_least_two_moves_made {
            let rolled_back = rollback(&game_state);
            reg.xadd.xadd(&StreamOutput::LOG(rolled_back))?;
            reg.xadd.xadd(&StreamOutput::MU(MoveUndone {
                game_id: um.game_id.clone(),
                player: um.player,
                game_state,
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
        board: compute_board(&game_state.moves),
        captures: compute_captures(&game_state.moves),
    }
}

fn compute_captures(moves: &[MoveMade]) -> Captures {
    todo!()
}
fn compute_board(moves: &[MoveMade]) -> Board {
    todo!()
}

use crate::repo::RepoErr;

#[derive(Debug)]
enum UndoProcessingErr {
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
