use crate::{
    consumer_group::{ConsumerGroup, Group, Message},
    XId,
};
use anyhow::{Context, Result};
use redis::{streams::StreamReadReply, Commands, Connection};
use std::collections::HashMap;

pub struct SortedStreams<'a, F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    pub consumer_groups: Vec<ConsumerGroup<F>>,
    pub timeout: usize,
    pub redis: &'a mut Connection,
}

impl<'a, F> SortedStreams<'a, F>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    /// "xreadgroup >" across streams, handle all the messages in time order, and acknowledge them all
    pub fn consume(&mut self) -> Result<()> {
        let unacked = Unacknowledged::default();
        for _consumer_group in &self.consumer_groups {
            todo!()
        }

        for ((stream, group), xids) in unacked.0 {
            let ids: Vec<String> = xids.iter().map(|xid| xid.to_string()).collect();
            self.redis.xack(&stream, &group.group_name, &ids)?
        }

        Ok(())
    }
}

/// Track unacknowledged messages by stream name
struct Unacknowledged(pub HashMap<(String, Group), Vec<XId>>);
impl Default for Unacknowledged {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
