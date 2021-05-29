//! Defines XREADGROUP / XACK streaming logic tied to a redis connection.
use super::XId;
use anyhow::Result;
use redis::{Commands, Connection, Value};
use std::collections::HashMap;

/// Handles connection to Redis and consumes messages from an individual stream.
/// Uses XREADGROUP only, never XREAD.
pub struct StreamHandler<F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    pub count: Option<usize>,
    pub handled_messages: u32,
    pub handler: F,
    pub stream: String,
}

impl<F> StreamHandler<F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    /// Calls xgroup_create_mkstream on the given stream name and returns this struct.
    pub fn init_redis_stream(
        stream: &str,
        handler: F,
        opts: ConsumerGroupOpts,
        redis: &mut Connection,
    ) -> Result<Self> {
        redis.xgroup_create_mkstream(stream, &opts.group.group_name, "$")?;
        Ok(StreamHandler {
            count: opts.count,
            handled_messages: 0,
            stream: stream.to_string(),
            handler,
        })
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

#[derive(Clone, Debug)]
pub enum StartPosition {
    EndOfStream,
    Other(String),
    StartOfStream,
}

#[derive(Debug)]
pub struct Group {
    pub group_name: String,
    pub consumer_name: String,
}

#[derive(Debug)]
pub struct ConsumerGroupOpts {
    pub count: Option<usize>,
    pub group: Group,
    pub block_ms: usize,
}

impl ConsumerGroupOpts {
    pub fn new(group: Group) -> Self {
        Self {
            count: None,
            group,
            block_ms: 5_000,
        }
    }

    /// Maximum number of message to read from the stream in one batch
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    /// Name of the group and consumer. Enables Redis group consumer behavior if
    /// specified
    pub fn group(mut self, group_name: &str, consumer_name: &str) -> Self {
        self.group = Group {
            group_name: group_name.to_string(),
            consumer_name: consumer_name.to_string(),
        };
        self
    }

    /// Maximum ms duration to block waiting for messages.
    pub fn block_ms(mut self, timeout_ms: usize) -> Self {
        self.block_ms = timeout_ms;
        self
    }
}
