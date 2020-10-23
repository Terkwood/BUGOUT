use super::xack::XAck;
use super::StreamData;
use log::error;
use redis::Client;
use redis_streams::XReadEntryId;

pub struct Unacknowledged {
    move_made: Vec<XReadEntryId>,
    history_provided: Vec<XReadEntryId>,
    sync_reply: Vec<XReadEntryId>,
    wait_for_opponent: Vec<XReadEntryId>,
    game_ready: Vec<XReadEntryId>,
    private_game_rejected: Vec<XReadEntryId>,
    colors_chosen: Vec<XReadEntryId>,
    bot_attached: Vec<XReadEntryId>,
}

const INIT_ACK_CAPACITY: usize = 25;
impl Unacknowledged {
    pub fn ack_all(&mut self, stream: &dyn XAck) {
        if !self.move_made.is_empty() {
            if let Err(_e) = stream.ack_move_made(&self.move_made) {
                error!("ack for move made failed")
            } else {
                self.move_made.clear();
            }
        }
        todo!("others")
    }
    pub fn push(&mut self, xid: XReadEntryId, event: StreamData) {
        match event {
            StreamData::MoveMade(_) => self.move_made.push(xid),
            StreamData::HistoryProvided(_) => self.history_provided.push(xid),
            StreamData::SyncReply(_) => self.sync_reply.push(xid),
            StreamData::WaitForOpponent(_) => self.wait_for_opponent.push(xid),
            StreamData::GameReady(_) => self.game_ready.push(xid),
            StreamData::PrivGameRejected(_) => self.private_game_rejected.push(xid),
            StreamData::BotAttached(_) => self.bot_attached.push(xid),
            StreamData::ColorsChosen(_) => self.colors_chosen.push(xid),
        }
    }
}

impl Default for Unacknowledged {
    fn default() -> Self {
        fn nv() -> Vec<XReadEntryId> {
            Vec::with_capacity(INIT_ACK_CAPACITY)
        }
        Self {
            move_made: nv(),
            history_provided: nv(),
            sync_reply: nv(),
            wait_for_opponent: nv(),
            game_ready: nv(),
            private_game_rejected: nv(),
            colors_chosen: nv(),
            bot_attached: nv(),
        }
    }
}
