use anyhow::{Context, Result};
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Commands, Connection, RedisResult, Value};
use std::collections::HashMap;

pub type Message = HashMap<String, Value>;

// A Consumer or Group Consumer handling connection to Redis and able to consume
// messages.
pub struct Consumer<'a, F>
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
