use super::topics;
use super::StreamInput;
use crate::Components;
use log::error;
use redis::{Client, Commands};
//use redis_streams::XReadEntryId;

pub trait XAck {
    //eg  ... fn ack_find_public_game(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
}

impl XAck for std::rc::Rc<Client> {}
