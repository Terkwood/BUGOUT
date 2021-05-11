use super::undo::consume_undo;
use super::*;
use crate::repo::Botness;
use crate::Components;
use log::error;
use redis_streams::XReadEntryId;

pub fn process(reg: &Components) {
    let mut unacked = Unacknowledged::default();
    loop {
        match reg.xread.xread_sorted() {
            Ok(xrr) => {
                for (xid, data) in xrr {
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
