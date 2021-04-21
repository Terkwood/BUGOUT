use super::topics::*;
use super::GROUP_NAME;
use log::{error, warn};
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Client, Commands};
use undo_model::api::*;
//use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::rc::Rc;

const BLOCK_MS: usize = 5000;

/// xread_sorted performs a redis xread then sorts the results
///
/// entry_ids: the minimum entry ids from which to read
pub trait XRead {
    // e.g.   TODO   fn xread_sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, XReadErr>;
}

#[derive(Debug)]
pub enum XReadErr {
    Deser(XReadDeserErr),
    Other,
}

#[derive(Debug)]
pub enum XReadDeserErr {
    EIDFormat,
    DataDeser(String),
}

const READ_OP: &str = ">";

impl XRead for Rc<Client> {}
