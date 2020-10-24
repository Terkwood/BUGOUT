use redis::Client;
use redis_streams::XReadEntryId;
use std::sync::Arc;

pub trait XAck: Send + Sync {
    fn ack_attach_bot(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckError>;
    fn ack_game_states_changelog(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckError>;
}
pub struct StreamAckError;

impl XAck for Arc<Client> {
    fn ack_attach_bot(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckError> {
        todo!()
    }

    fn ack_game_states_changelog(&self, xids: &[XReadEntryId]) -> Result<(), StreamAckError> {
        todo!()
    }
}
