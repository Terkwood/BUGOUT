//! Defines consumer groups and streaming logic.

use anyhow::{Context, Result};
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Commands, Connection, RedisResult, Value};
use std::collections::HashMap;

pub type Message = HashMap<String, Value>;

/// Handles connection to Redis and consumes messages from an individual stream.
/// Uses XREADGROUP only, never XREAD.
pub struct ConsumerGroup<'a, F>
where
    F: FnMut(&str, &Message) -> Result<()>,
{
    pub count: Option<usize>,
    pub group: (String, String),
    pub handled_messages: u32,
    pub handler: F,
    pub next_pos: String,
    pub process_pending: bool,
    pub redis: &'a mut Connection,
    pub stream: String,
    pub timeout: usize,
}

impl<'a, F> ConsumerGroup<'a, F>
where
    F: FnMut(&str, &Message) -> Result<()>,
{
    /// Initializes a new `stream::Consumer`.
    pub fn init(
        redis: &'a mut Connection,
        stream: &str,
        handler: F,
        opts: ConsumerGroupOpts,
    ) -> Result<Self> {
        todo!()
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
    pub process_pending: bool,
    pub start_pos: StartPosition,
    pub timeout: usize,
}

impl ConsumerGroupOpts {
    pub fn new(group: Group) -> Self {
        Self {
            count: None,
            create_stream_if_not_exists: true,
            group,
            process_pending: true,
            start_pos: StartPosition::EndOfStream,
            timeout: 2_000,
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

    /// Start by processing pending messages before switching to real time data
    /// (default: `true`)
    pub fn process_pending(mut self, process_pending: bool) -> Self {
        self.process_pending = process_pending;
        self
    }

    /// Where to start reading messages in the stream.
    pub fn start_pos(mut self, start_pos: StartPosition) -> Self {
        self.start_pos = start_pos;
        self
    }

    /// Maximum ms duration to block waiting for messages.
    pub fn timeout(mut self, timeout: usize) -> Self {
        self.timeout = timeout;
        self
    }
}
