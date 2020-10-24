use super::topics;
use redis::{Client, Commands};
use redis_streams::XReadEntryId;
use std::sync::Arc;

pub trait XAck: Send + Sync {
    fn ack_attach_bot(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckError>;
    fn ack_game_states_changelog(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckError>;
}
pub struct StreamAckError;

impl XAck for Arc<Client> {
    fn ack_attach_bot(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckError> {
        ack(self, topics::ATTACH_BOT_CMD, xids)
    }

    fn ack_game_states_changelog(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckError> {
        ack(self, topics::GAME_STATES_CHANGELOG, xids)
    }
}

fn ack(client: &Client, key: &str, ids: &[XReadEntryId]) -> Result<(), StreamAckError> {
    match client.get_connection() {
        Ok(mut conn) => {
            let idstrs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
            let _: usize = conn.xack(key, super::GROUP_NAME, &idstrs)?;
            Ok(())
        }
        Err(_) => Err(StreamAckError),
    }
}

impl From<redis::RedisError> for StreamAckError {
    fn from(_: redis::RedisError) -> Self {
        Self
    }
}
