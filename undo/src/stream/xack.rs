use super::topics;
use super::StreamInput;
use crate::Components;
use log::error;
use redis::{Client, Commands};
use redis_streams::XReadEntryId;

pub trait XAck {
    fn ack_changelog(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_bot_attached(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_undo_move(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
}

pub struct StreamAckErr;

pub struct Unacknowledged {
    changelog: Vec<XReadEntryId>,
    undo_move: Vec<XReadEntryId>,
    bot_attached: Vec<XReadEntryId>,
}

impl XAck for std::rc::Rc<Client> {
    fn ack_changelog(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::GAME_STATES_CHANGELOG, xids)
    }
    fn ack_bot_attached(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::BOT_ATTACHED, xids)
    }
    fn ack_undo_move(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::UNDO_MOVE, xids)
    }
}

fn ack(client: &Client, key: &str, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
    match client.get_connection() {
        Ok(mut conn) => {
            let idstrs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
            let _: usize = conn.xack(key, super::GROUP_NAME, &idstrs)?;
            Ok(())
        }
        Err(_) => Err(StreamAckErr),
    }
}

impl From<redis::RedisError> for StreamAckErr {
    fn from(_: redis::RedisError) -> Self {
        Self
    }
}

impl Unacknowledged {
    pub fn ack_all(&mut self, reg: &Components) {
        if !self.changelog.is_empty() {
            if let Err(_e) = reg.xack.ack_changelog(&self.changelog) {
                error!("ack for changelog failed")
            } else {
                self.changelog.clear();
            }
        }

        if !self.undo_move.is_empty() {
            if let Err(_e) = reg.xack.ack_undo_move(&self.undo_move) {
                error!("ack for undo move failed")
            } else {
                self.undo_move.clear();
            }
        }
        if !self.bot_attached.is_empty() {
            if let Err(_e) = reg.xack.ack_bot_attached(&self.bot_attached) {
                error!("ack for bot attached failed")
            } else {
                self.bot_attached.clear();
            }
        }
    }

    pub fn push(&mut self, xid: XReadEntryId, event: StreamInput) {
        match event {
            StreamInput::UM(_) => self.undo_move.push(xid),
            StreamInput::BA(_) => self.bot_attached.push(xid),
            StreamInput::LOG(_) => self.changelog.push(xid),
        }
    }
}

const INIT_ACK_CAPACITY: usize = 50;
impl Default for Unacknowledged {
    fn default() -> Self {
        Self {
            changelog: Vec::with_capacity(INIT_ACK_CAPACITY),
            undo_move: Vec::with_capacity(INIT_ACK_CAPACITY),
            bot_attached: Vec::with_capacity(INIT_ACK_CAPACITY),
        }
    }
}
