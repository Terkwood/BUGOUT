//! Defines consumer groups and streaming logic.

use super::XId;
use anyhow::{Context, Result};
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Commands, Connection, RedisResult, Value};
use std::collections::HashMap;

/// Handles connection to Redis and consumes messages from an individual stream.
/// Uses XREADGROUP only, never XREAD.
pub struct ConsumerGroup<F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    pub count: Option<usize>,
    pub group: Group,
    pub handled_messages: u32,
    pub handler: F,
    pub stream: String,
}

impl<F> ConsumerGroup<F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    /// Initializes a new `stream::Consumer` and returns a ConsumerGroup struct.
    pub fn init(
        stream: &str,
        handler: F,
        opts: ConsumerGroupOpts,
        redis: &mut Connection,
    ) -> Result<Self> {
        redis.xgroup_create_mkstream(stream, &opts.group.group_name, "$")?;
        Ok(ConsumerGroup {
            count: opts.count,
            group: opts.group,
            handled_messages: 0,
            stream: stream.to_string(),
            handler,
        })
    }

    /// Process a message by calling the handler, returning the same XId
    /// passed to the handler.
    fn _handle_message(&mut self, xid: XId, message: &Message) -> Result<XId> {
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
    pub timeout_ms: usize,
}

impl ConsumerGroupOpts {
    pub fn new(group: Group) -> Self {
        Self {
            count: None,
            group,
            timeout_ms: 5_000,
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
    pub fn timeout(mut self, timeout_ms: usize) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}
