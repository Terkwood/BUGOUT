//! Wraps a a function which is used to process individual
//! messages from given stream, in time order.
use crate::*;
use anyhow::Result;
use redis::Value;
use std::collections::HashMap;

/// Handles connection to Redis and consumes messages from an individual stream.
/// Uses XREADGROUP only, never XREAD.
pub struct StreamHandler<F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    pub handled_messages: u32,
    pub handler: F,
    pub stream: String,
}

impl<F> StreamHandler<F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    pub fn new(stream: &str, handler: F) -> Self {
        StreamHandler {
            handled_messages: 0,
            stream: stream.to_string(),
            handler,
        }
    }

    /// Process a message by calling the handler, returning the same XId
    /// passed to the handler.
    pub fn handle_message(&mut self, xid: XId, message: &Message) -> Result<XId> {
        (self.handler)(xid, message)?; // Call handler
        self.handled_messages += 1;
        Ok(xid)
    }
}

pub type Message = HashMap<String, Value>;

#[derive(Debug, Clone)]
pub struct Group {
    pub group_name: String,
    pub consumer_name: String,
}
