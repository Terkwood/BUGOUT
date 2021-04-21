use super::topics;
use super::StreamInput;
use crate::Components;
use log::error;
use redis::{Client, Commands};
//use redis_streams::XReadEntryId;

pub trait XAck {}

impl XAck for std::rc::Rc<Client> {}
