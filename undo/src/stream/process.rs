use super::*;
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
        StreamInput::LOG(_) => todo!(),
        StreamInput::BA(ba) => consume_ba(ba, reg),
        StreamInput::UM(_) => todo!(),
    }
}

fn consume_ba(ba: &BotAttached, reg: &Components) {
    if let Err(e) = reg.bot_repo.put(&ba.game_id, ba.player) {
        error!("could not track bot attached: {:?}", e)
    }
}
