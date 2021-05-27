//! Defines consumer groups and streaming logic.

use super::XId;
use anyhow::{Context, Result};
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Commands, Connection, RedisResult, Value};
use std::collections::HashMap;

pub type Message = HashMap<String, Value>;

/// Handles connection to Redis and consumes messages from an individual stream.
/// Uses XREADGROUP only, never XREAD.
pub struct ConsumerGroup<'a, F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    pub count: Option<usize>,
    pub group: Group,
    pub handled_messages: u32,
    pub handler: F,
    pub redis: &'a mut Connection,
    pub stream: String,
    pub timeout: usize,
}

impl<'a, F> ConsumerGroup<'a, F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    /// Initializes a new `stream::Consumer`.
    pub fn init(
        _redis: &'a mut Connection,
        _stream: &str,
        _handler: F,
        _opts: ConsumerGroupOpts,
    ) -> Result<Self> {
        todo!()
    }

    /// Process a message by calling the handler, returning the same XId
    /// passed to the handler.
    fn handle_message(&mut self, xid: XId, message: &Message) -> Result<XId> {
        (self.handler)(xid, message)?; // Call handler
        self.handled_messages += 1;
        Ok(xid)
    }

    /// Acknowledge messages by ID
    fn ack_messages(&mut self, xids: &[XId]) -> Result<()> {
        let ids: Vec<String> = xids.iter().map(|xid| xid.to_string()).collect();
        Ok(self
            .redis
            .xack(&self.stream, &self.group.group_name, &ids)?)
    }
}

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
    pub create_stream_if_not_exists: bool,
    pub group: Group,
    pub timeout_ms: usize,
}

impl ConsumerGroupOpts {
    pub fn new(group: Group) -> Self {
        Self {
            count: None,
            create_stream_if_not_exists: true,
            group,
            timeout_ms: 5_000,
        }
    }

    /// Maximum number of message to read from the stream in one batch
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    /// Create the stream in Redis before registering the group (default: `true`).
    pub fn create_stream_if_not_exists(mut self, create_stream_if_not_exists: bool) -> Self {
        self.create_stream_if_not_exists = create_stream_if_not_exists;
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
    pub fn timeout(mut self, timeout_ms: usize) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}
