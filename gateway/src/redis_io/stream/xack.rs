use crate::topics;
use redis::{Client, Commands};
use redis_streams::XReadEntryId;
use std::sync::Arc;

pub trait XAck {
    fn ack_move_made(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_history_provided(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_sync_reply(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_wait_for_opponent(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_game_ready(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_private_game_rejected(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_bot_attached(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_colors_chosen(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_move_undone(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_undo_rejected(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
}

#[derive(Debug)]
pub struct StreamAckErr;

impl XAck for Arc<Client> {
    fn ack_move_made(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::MOVE_MADE_TOPIC, ids)
    }

    fn ack_history_provided(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::HISTORY_PROVIDED_TOPIC, ids)
    }

    fn ack_sync_reply(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::SYNC_REPLY_TOPIC, ids)
    }

    fn ack_wait_for_opponent(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::WAIT_FOR_OPPONENT_TOPIC, ids)
    }

    fn ack_game_ready(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::GAME_READY_TOPIC, ids)
    }

    fn ack_private_game_rejected(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::PRIVATE_GAME_REJECTED_TOPIC, ids)
    }

    fn ack_bot_attached(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::BOT_ATTACHED_TOPIC, ids)
    }

    fn ack_colors_chosen(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::COLORS_CHOSEN_TOPIC, ids)
    }

    fn ack_move_undone(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::MOVE_UNDONE_TOPIC, ids)
    }

    fn ack_undo_rejected(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::UNDO_REJECTED_TOPIC, ids)
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
