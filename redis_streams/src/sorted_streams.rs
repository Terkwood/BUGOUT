use crate::{
    stream_handler::{Group, Message, StreamHandler},
    XId,
};
use anyhow::{Context, Result};
use redis::{
    streams::{StreamReadOptions, StreamReadReply},
    Commands, Connection,
};
use std::collections::HashMap;

pub struct SortedStreams<'a, F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    pub handlers: Vec<StreamHandler<F>>,
    pub group: Group,
    pub timeout: usize,
    pub redis: &'a mut Connection,
}

const READ_OP: &str = ">";
impl<'a, F> SortedStreams<'a, F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    /// "xreadgroup >" across streams, handle all the messages in time order, and acknowledge them all
    pub fn consume(&mut self) -> Result<()> {
        let unacked = Unacknowledged::default();
        let opts = StreamReadOptions::default()
            .block(self.timeout)
            .group(&self.group.group_name, &self.group.consumer_name);
        let stream_names: Vec<String> =
            self.handlers.iter().map(|h| h.stream.to_string()).collect();
        let read_ops: Vec<String> = stream_names.iter().map(|_| READ_OP.to_string()).collect();

        let _xrr: StreamReadReply = self.redis.xread_options(&stream_names, &read_ops, opts)?;

        for _consumer_group in &self.handlers {
            todo!()
        }

        for (stream, xids) in unacked.0 {
            let ids: Vec<String> = xids.iter().map(|xid| xid.to_string()).collect();
            self.redis.xack(&stream, &self.group.group_name, &ids)?
        }

        Ok(())
    }
}

/// Track unacknowledged messages by stream name
struct Unacknowledged(pub HashMap<String, Vec<XId>>);
impl Default for Unacknowledged {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
