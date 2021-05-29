use crate::*;
use anyhow::Result;
use redis::{
    streams::{StreamReadOptions, StreamReadReply},
    Commands, Connection,
};
use std::collections::HashMap;
use std::str::FromStr;

pub trait SortedStreams {
    /// "XREADGROUP >" across streams, handle all the messages in time order,
    /// and acknowledge them all.  The XACK calls happen once per stream,
    /// acknowleding all XIds for that stream at once.  The XACK calls happen
    /// after _all_ individual message handlers have been invoked.
    fn consume(&mut self) -> Result<()>;
}

pub struct RedisSortedStreams<'a, F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    pub handlers: HashMap<String, StreamHandler<F>>,
    pub group: Group,
    pub block_ms: usize,
    pub redis: &'a mut Connection,
}

impl<'a, F> RedisSortedStreams<'a, F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    /// Calls xgroup_create_mkstream on the given stream name and returns this struct.
    pub fn xgroup_create_mkstreams(
        stream_handlers: Vec<(&str, F)>,
        opts: &ConsumerGroupOpts,
        redis: &'a mut Connection,
    ) -> Result<Self> {
        let mut handlers = HashMap::new();
        for (stream, handler) in stream_handlers {
            let _: Result<(), redis::RedisError> = match redis
                .xgroup_create_mkstream(stream, &opts.group.group_name, "$")
                .map(|_: redis::Value| ())
            {
                Err(err) => {
                    if err.to_string() == "BUSYGROUP: Consumer Group name already exists" {
                        Ok(()) // ignore busygroup
                    } else {
                        Err(err)
                    }
                }
                _ => Ok(()),
            };
            handlers.insert(stream.to_string(), StreamHandler::new(stream, handler));
        }

        Ok(Self {
            group: opts.group.clone(),
            redis,
            block_ms: opts.block_ms,
            handlers,
        })
    }
}

impl<'a, F> SortedStreams for RedisSortedStreams<'a, F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    fn consume(&mut self) -> Result<()> {
        let mut unacked = Unacknowledged::default();
        let opts = StreamReadOptions::default()
            .block(self.block_ms)
            .group(&self.group.group_name, &self.group.consumer_name);
        let stream_names: Vec<String> = self.handlers.keys().cloned().collect();
        let read_ops: Vec<String> = stream_names.iter().map(|_| READ_OP.to_string()).collect();

        let xrr: StreamReadReply = self.redis.xread_options(&stream_names, &read_ops, opts)?;
        let mut by_time: Vec<(XId, StreamMessage)> = Vec::with_capacity(50);

        for k in xrr.keys {
            let key = k.key;
            for x in k.ids {
                if let Ok(xid) = XId::from_str(&x.id) {
                    by_time.push((xid, StreamMessage(key.clone(), x.map)));
                }
            }
        }

        by_time.sort_by_key(|t| t.0);

        for (xid, StreamMessage(stream, message)) in by_time {
            if let Some(handler) = self.handlers.get_mut(&stream) {
                handler.handle_message(xid, &message)?;
                if let Some(unacked_for_stream) = unacked.0.get_mut(&stream) {
                    unacked_for_stream.push(xid);
                }
            }
        }

        for (stream, xids) in unacked.0 {
            let ids: Vec<String> = xids.iter().map(|xid| xid.to_string()).collect();
            self.redis.xack(&stream, &self.group.group_name, &ids)?
        }

        Ok(())
    }
}

const READ_OP: &str = ">";

struct StreamMessage(pub String, pub Message);

/// Track unacknowledged messages by stream name
struct Unacknowledged(pub HashMap<String, Vec<XId>>);
impl Default for Unacknowledged {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
