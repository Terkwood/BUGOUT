use crate::*;
use anyhow::Result;
use redis::{
    streams::{StreamReadOptions, StreamReadReply},
    Commands,
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

use redis::Client;
use std::rc::Rc;
pub struct RedisSortedStreams<F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    pub group: Group,
    pub block_ms: usize,
    pub redis: Rc<Client>,
    pub guts: SortedStreamGuts<F>,
}

impl<F> RedisSortedStreams<F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    /// Calls xgroup_create_mkstream on the given stream name and returns this struct.
    pub fn xgroup_create_mkstreams(
        stream_handlers: Vec<(&str, F)>,
        opts: &ConsumerGroupOpts,
        redis: Rc<Client>,
    ) -> Result<Self> {
        let mut handlers = HashMap::new();
        for (stream, handler) in stream_handlers {
            let mut conn = redis.get_connection()?;
            let _: Result<(), redis::RedisError> = match conn
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

        let block_ms = opts.block_ms;

        let xrgs = Box::new(RedisXReadGroupSorted {
            redis: redis.clone(),
            group: opts.group.clone(),
            block_ms,
        });
        let guts = SortedStreamGuts {
            handlers,
            xack: Box::new(RedisXAck {
                redis: redis.clone(),
                group: opts.group.clone(),
            }),
            xreadgroup_sorted: xrgs,
        };

        Ok(Self {
            group: opts.group.clone(),
            redis: redis.clone(),
            block_ms: opts.block_ms,
            guts: guts,
        })
    }
}

pub struct SortedStreamGuts<F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    pub handlers: HashMap<String, StreamHandler<F>>,
    pub xreadgroup_sorted: Box<dyn XReadGroupSorted>,
    pub xack: Box<dyn XAck>,
}

pub trait XReadGroupSorted {
    fn read(&mut self, stream_names: &[String]) -> Result<Vec<(XId, StreamMessage)>>;
}
pub trait XAck {
    fn ack(&mut self, stream_name: &str, ids: &[String]) -> Result<()>;
}

struct RedisXReadGroupSorted {
    pub group: Group,
    pub block_ms: usize,
    pub redis: Rc<Client>,
}
struct RedisXAck {
    pub group: Group,
    pub redis: Rc<Client>,
}
impl XAck for RedisXAck {
    fn ack(&mut self, stream_name: &str, ids: &[String]) -> Result<()> {
        self.redis
            .get_connection()?
            .xack(stream_name, &self.group.group_name, ids)
            .map_err(|_| anyhow::Error::msg("xack"))
    }
}

impl XReadGroupSorted for RedisXReadGroupSorted {
    fn read(&mut self, stream_names: &[String]) -> Result<Vec<(XId, StreamMessage)>> {
        Ok({
            let mut out = Vec::with_capacity(50);
            let opts = StreamReadOptions::default()
                .block(self.block_ms)
                .group(&self.group.group_name, &self.group.consumer_name);
            let read_ops: Vec<String> = stream_names.iter().map(|_| READ_OP.to_string()).collect();

            let mut redis = self.redis.get_connection()?;

            let xrr: StreamReadReply = redis.xread_options(&stream_names, &read_ops, opts)?;
            for k in xrr.keys {
                let key = k.key;
                for x in k.ids {
                    let message: Message = x.map;
                    if let Ok(xid) = XId::from_str(&x.id) {
                        out.push((xid, StreamMessage(key.clone(), message)));
                    }
                }
            }
            out.sort_by_key(|t| t.0);
            out
        })
    }
}

impl<F> SortedStreams for SortedStreamGuts<F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    fn consume(&mut self) -> Result<()> {
        let mut unacked = Unacknowledged::default();
        let stream_names: Vec<String> = self.handlers.keys().cloned().collect();

        let by_time: Vec<(XId, StreamMessage)> = self.xreadgroup_sorted.read(&stream_names)?;

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
            self.xack.ack(&stream, &ids)?
        }

        Ok(())
    }
}

impl<'a, F> SortedStreams for RedisSortedStreams<F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    fn consume(&mut self) -> Result<()> {
        self.guts.consume()
    }
}

const READ_OP: &str = ">";

#[derive(Clone)]
pub struct StreamMessage(pub String, pub Message);

/// Track unacknowledged messages by stream name
struct Unacknowledged(pub HashMap<String, Vec<XId>>);
impl Default for Unacknowledged {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
