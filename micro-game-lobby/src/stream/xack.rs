use crate::components::Components;
use crate::stream::StreamInput;
use log::error;
use redis::{Client, Commands};
use redis_streams::XReadEntryId;

pub trait XAck {
    fn ack_find_public_game(&self, xid: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_join_priv_game(&self, xid: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_create_game(&self, xid: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_session_disconnected(&self, xid: &[XReadEntryId]) -> Result<(), StreamAckErr>;
}

impl XAck for std::rc::Rc<Client> {
    fn ack_find_public_game(&self, xid: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        todo!()
    }

    fn ack_join_priv_game(&self, xid: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        todo!()
    }

    fn ack_create_game(&self, xid: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        todo!()
    }

    fn ack_session_disconnected(&self, xid: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        todo!()
    }
}

fn nah_xack(
    key: &str,
    group: &str,
    ids: &[XReadEntryId],
    client: &Client,
) -> Result<(), redis::RedisError> {
    let c = client.get_connection();
    if let Ok(mut conn) = c {
        let idstrs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let _: usize = conn.xack(key, group, &idstrs)?;
        Ok(())
    } else {
        c.map(|_| ())
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
    pub fn ack_all(&mut self, xack: &dyn XAck) {
        if !self.fpg.is_empty() {
            if let Err(_e) = xack.ack_find_public_game(&self.fpg) {
                error!("ack for fpg failed")
            } else {
                self.fpg.clear();
            }
        }

        if !self.jpg.is_empty() {
            if let Err(_e) = xack.ack_join_priv_game(&self.jpg) {
                error!("ack for jpg failed")
            } else {
                self.jpg.clear();
            }
        }
        if !self.cg.is_empty() {
            if let Err(_e) = xack.ack_create_game(&self.cg) {
                error!("ack for create game failed")
            } else {
                self.cg.clear();
            }
        }
        if !self.sd.is_empty() {
            if let Err(_e) = xack.ack_session_disconnected(&self.sd) {
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
    let mut conn = client.get_connection().expect("conn");
    let idstrs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
    let _: usize = conn.xack(key, super::GROUP_NAME, &idstrs)?;
    Ok(())
}

impl From<redis::RedisError> for StreamAckErr {
    fn from(_: redis::RedisError) -> Self {
        Self
    }
}
