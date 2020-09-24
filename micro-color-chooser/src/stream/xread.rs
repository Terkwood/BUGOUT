use log::error;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Client, Commands};
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;

pub trait XRead {
    //fn xread_sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, StreamReadErr>;
}

impl XRead for Rc<Client> {}
