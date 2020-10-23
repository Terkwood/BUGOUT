use crate::topics;
use redis::{Client, Commands};
use redis_streams::XReadEntryId;
use std::sync::Arc;

pub trait XAck {
    fn ack_move_made(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
}

#[derive(Debug)]
pub struct StreamAckErr;

impl XAck for Arc<Client> {
    fn ack_move_made(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::MOVE_MADE_TOPIC, ids)
    }
}

fn ack(client: &Client, key: &str, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
    if let Ok(mut conn) = client.get_connection() {
        let idstrs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let _: usize = conn.xack(key, super::GROUP_NAME, &idstrs)?;
        Ok(())
    } else {
        Err(StreamAckErr)
    }
}

impl From<redis::RedisError> for StreamAckErr {
    fn from(_: redis::RedisError) -> Self {
        Self
    }
}
