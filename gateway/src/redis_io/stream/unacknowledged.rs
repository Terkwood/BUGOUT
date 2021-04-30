use super::xack::XAck;
use super::StreamData;
use log::error;
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
    move_undone: Vec<XReadEntryId>,
    undo_rejected: Vec<XReadEntryId>,
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
        if !self.history_provided.is_empty() {
            if let Err(_e) = stream.ack_history_provided(&self.history_provided) {
                error!("ack hp failed")
            } else {
                self.history_provided.clear();
            }
        }
        if !self.sync_reply.is_empty() {
            if let Err(_e) = stream.ack_sync_reply(&self.sync_reply) {
                error!("ack sync_reply failed")
            } else {
                self.sync_reply.clear();
            }
        }
        if !self.wait_for_opponent.is_empty() {
            if let Err(_e) = stream.ack_wait_for_opponent(&self.wait_for_opponent) {
                error!("ack wait_for_opponent failed")
            } else {
                self.wait_for_opponent.clear();
            }
        }

        if !self.game_ready.is_empty() {
            if let Err(_e) = stream.ack_game_ready(&self.game_ready) {
                error!("ack game_ready failed")
            } else {
                self.game_ready.clear();
            }
        }
        if !self.private_game_rejected.is_empty() {
            if let Err(_e) = stream.ack_private_game_rejected(&self.private_game_rejected) {
                error!("ack private_game_rejected failed")
            } else {
                self.private_game_rejected.clear();
            }
        }
        if !self.colors_chosen.is_empty() {
            if let Err(_e) = stream.ack_colors_chosen(&self.colors_chosen) {
                error!("ack colors_chosen failed")
            } else {
                self.colors_chosen.clear();
            }
        }
        if !self.bot_attached.is_empty() {
            if let Err(_e) = stream.ack_bot_attached(&self.bot_attached) {
                error!("ack bot_attached failed")
            } else {
                self.bot_attached.clear();
            }
        }

        if !self.move_undone.is_empty() {
            if let Err(_) = stream.ack_move_undone(&self.move_undone) {
                error!("ack move undone failed")
            } else {
                self.move_undone.clear();
            }
        }

        if !self.undo_rejected.is_empty() {
            if let Err(_) = stream.ack_undo_rejected(&self.undo_rejected) {
                error!("ack undo rejected failed")
            } else {
                self.undo_rejected.clear();
            }
        }
    }
    pub fn push(&mut self, xid: XReadEntryId, event: StreamData) {
        match event {
            StreamData::MoveMade(_) => self.move_made.push(xid),
            StreamData::HistoryProvided(_) => self.history_provided.push(xid),
            StreamData::SyncReply(_) => self.sync_reply.push(xid),
            StreamData::WaitForOpponent(_) => self.wait_for_opponent.push(xid),
            StreamData::GameReady(_) => self.game_ready.push(xid),
            StreamData::PrivGameRejected(_) => self.private_game_rejected.push(xid),
            StreamData::ColorsChosen(_) => self.colors_chosen.push(xid),
            StreamData::BotAttached(_) => self.bot_attached.push(xid),
            StreamData::MoveUndone(_) => self.move_undone.push(xid),
            StreamData::UndoRejected(_) => self.undo_rejected.push(xid),
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
            move_undone: nv(),
            undo_rejected: nv(),
        }
    }
}
