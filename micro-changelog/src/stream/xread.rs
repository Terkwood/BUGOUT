use super::StreamTopics;
use crate::model::*;
use crate::redis;
use crate::repo::entry_id_repo::AllEntryIds;
use micro_model_moves::{GameId, GameState};
use redis_conn_pool::Pool;
use redis_streams::XReadEntryId;

const BLOCK_MSEC: u32 = 5000;

pub fn xread_sorted(
    entry_ids: AllEntryIds,
    topics: &StreamTopics,
    pool: &Pool,
) -> Result<Vec<(XReadEntryId, StreamData)>, redis::RedisError> {
    todo!()
}

#[derive(Clone)]
pub enum StreamData {
    MA(MoveAcceptedEvent),
    GS(GameId, GameState),
    GR(GameReadyEvent),
}
