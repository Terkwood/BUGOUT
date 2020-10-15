use crate::stream::StreamInput;
use crate::topics;
use crate::Components;
use log::error;
use redis::{Client, Commands};
use redis_streams::XReadEntryId;

pub trait XAck {
    fn ack_find_public_game(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_join_priv_game(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_create_game(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_session_disconnected(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
}

impl XAck for std::rc::Rc<Client> {
    fn ack_find_public_game(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::FIND_PUBLIC_GAME, xids)
    }

    fn ack_join_priv_game(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::JOIN_PRIVATE_GAME, xids)
    }

    fn ack_create_game(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::CREATE_GAME, xids)
    }

    fn ack_session_disconnected(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::SESSION_DISCONNECTED, xids)
    }
}

#[derive(Debug)]
pub struct StreamAckErr;

pub struct Unacknowledged {
    fpg: Vec<XReadEntryId>,
    jpg: Vec<XReadEntryId>,
    cg: Vec<XReadEntryId>,
    sd: Vec<XReadEntryId>,
}

impl Unacknowledged {
    pub fn ack_all(&mut self, reg: &Components) {
        if !self.fpg.is_empty() {
            if let Err(_e) = reg.xack.ack_find_public_game(&self.fpg) {
                error!("ack for fpg failed")
            } else {
                self.fpg.clear();
            }
        }

        if !self.jpg.is_empty() {
            if let Err(_e) = reg.xack.ack_join_priv_game(&self.jpg) {
                error!("ack for jpg failed")
            } else {
                self.jpg.clear();
            }
        }
        if !self.cg.is_empty() {
            if let Err(_e) = reg.xack.ack_create_game(&self.cg) {
                error!("ack for create game failed")
            } else {
                self.cg.clear();
            }
        }
        if !self.sd.is_empty() {
            if let Err(_e) = reg.xack.ack_session_disconnected(&self.sd) {
                error!("ack for session disconn failed")
            } else {
                self.sd.clear();
            }
        }
    }
    pub fn push(&mut self, xid: XReadEntryId, event: StreamInput) {
        match event {
            StreamInput::FPG(_) => self.fpg.push(xid),
            StreamInput::JPG(_) => self.jpg.push(xid),
            StreamInput::CG(_) => self.cg.push(xid),
            StreamInput::SD(_) => self.sd.push(xid),
        }
    }
}

const INIT_ACK_CAPACITY: usize = 50;
impl Default for Unacknowledged {
    fn default() -> Self {
        Self {
            fpg: Vec::with_capacity(INIT_ACK_CAPACITY),
            jpg: Vec::with_capacity(INIT_ACK_CAPACITY),
            cg: Vec::with_capacity(INIT_ACK_CAPACITY),
            sd: Vec::with_capacity(INIT_ACK_CAPACITY),
        }
    }
}

fn ack(client: &Client, key: &str, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
    if let Ok(mut conn) = client.get_connection() {
        let idstrs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let _: usize = conn.xack(key, super::GROUP_NAME, &idstrs)?;
        Ok(())
    } else {
        Err(StreamAckErr)
    }
}

impl From<redis::RedisError> for StreamAckErr {
    fn from(_: redis::RedisError) -> Self {
        Self
    }
}
