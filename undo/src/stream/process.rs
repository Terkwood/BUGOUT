use super::*;
use crate::repo::Botness;
use crate::Components;
use log::{error, info};
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
        StreamInput::BA(bot_attached) => consume_ba(bot_attached, reg),
        StreamInput::UM(undo_move) => {
            if let Err(e) = consume_um(undo_move, reg) {
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

fn consume_ba(ba: &BotAttached, reg: &Components) {
    if let Err(e) = reg.botness_repo.put(&ba.game_id, ba.player, Botness::IsBot) {
        error!("could not track bot attached: {:?}", e)
    }
}

fn consume_um(um: &UndoMove, reg: &Components) -> Result<(), UndoProcessingErr> {
    let botness = reg.botness_repo.get(&um.game_id, um.player)?;
    let requester_is_human = botness == Botness::IsHuman;

    let current_move_is_human: bool =
        todo!("check that we are not waiting on a bot to finish their move");

    let not_the_very_beginning: bool =
        todo!("check that there is a move which can be undone  (first move fails)");

    if (requester_is_human && current_move_is_human && not_the_very_beginning) {
        todo!("emit a game_state event to the changelog stream");
        todo!("emit a move_undone event");
    } else {
        todo!("on fail: emit UndoMove  to a rejected stream");
    }

    Ok(())
}

use crate::repo::RepoErr;

#[derive(Debug)]
enum UndoProcessingErr {
    Repo(RepoErr),
}
impl From<crate::repo::RepoErr> for UndoProcessingErr {
    fn from(e: RepoErr) -> Self {
        Self::Repo(e)
    }
}
