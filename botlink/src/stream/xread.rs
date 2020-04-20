use super::topics;
use crate::repo::AllEntryIds;
use log::{trace, warn};
use micro_model_bot::gateway::AttachBot;
use micro_model_moves::{GameId, GameState};
use redis_conn_pool::redis;
use redis_conn_pool::Pool;
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

const BLOCK_MSEC: u32 = 5000;

pub type XReadResult = Vec<HashMap<String, Vec<HashMap<String, redis::Value>>>>;

/// xread_sorted performs a redis xread then sorts the results
///
/// entry_ids: the minimum entry ids from which to read
pub trait XReader: Send + Sync {
    fn xread_sorted(
        &self,
        entry_ids: AllEntryIds,
    ) -> Result<Vec<(XReadEntryId, StreamData)>, redis::RedisError>;
}

pub struct RedisXReader {
    pub pool: Arc<Pool>,
}
impl XReader for RedisXReader {
    fn xread_sorted(
        &self,
        entry_ids: AllEntryIds,
    ) -> Result<std::vec::Vec<(XReadEntryId, StreamData)>, redis::RedisError> {
        trace!(
            "xreading from {} and {}",
            topics::ATTACH_BOT_CMD,
            topics::GAME_STATES_CHANGELOG
        );
        let mut conn = self.pool.get().unwrap();
        let xrr = redis::cmd("XREAD")
            .arg("BLOCK")
            .arg(&BLOCK_MSEC.to_string())
            .arg("STREAMS")
            .arg(topics::ATTACH_BOT_CMD)
            .arg(topics::GAME_STATES_CHANGELOG)
            .arg(entry_ids.attach_bot_eid.to_string())
            .arg(entry_ids.game_states_eid.to_string())
            .query::<XReadResult>(&mut *conn)?;
        let unsorted = deser(xrr);
        let sorted_keys: Vec<XReadEntryId> = {
            let mut ks: Vec<XReadEntryId> = unsorted.keys().copied().collect();
            ks.sort();
            ks
        };
        let mut answer = vec![];
        for sk in sorted_keys {
            if let Some(data) = unsorted.get(&sk) {
                answer.push((sk, data.clone()))
            }
        }
        Ok(answer)
    }
}

#[derive(Clone, Debug)]
pub enum StreamData {
    AB(AttachBot),
    GS(GameId, GameState),
}

fn deser(xread_result: XReadResult) -> HashMap<XReadEntryId, StreamData> {
    let mut stream_data = HashMap::new();

    for hash in xread_result.iter() {
        for (xread_topic, xread_data) in hash.iter() {
            if &xread_topic[..] == topics::GAME_STATES_CHANGELOG {
                for with_timestamps in xread_data {
                    for (k, v) in with_timestamps {
                        let shape: Result<(String, String, String, Option<Vec<u8>>), _> = // game-id <uuidstr> data <bin>
                            redis::FromRedisValue::from_redis_value(&v);
                        if let Ok(s) = shape {
                            if let (Ok(seq_no), Some(game_id), Some(game_state)) = (
                                XReadEntryId::from_str(k),
                                Uuid::from_str(&s.1).ok(),
                                s.3.clone().and_then(|bytes| GameState::from(&bytes).ok()),
                            ) {
                                stream_data
                                    .insert(seq_no, StreamData::GS(GameId(game_id), game_state));
                            } else {
                                warn!("Xread: Deser error around game states data")
                            }
                        } else {
                            warn!("Fail XREAD")
                        }
                    }
                }
            } else if &xread_topic[..] == topics::ATTACH_BOT_CMD {
                for with_timestamps in xread_data {
                    for (k, v) in with_timestamps {
                        let shape: Result<(String, Vec<u8>), _> = // data <bin>
                            redis::FromRedisValue::from_redis_value(&v);
                        if let Ok(s) = shape {
                            if let (Ok(seq_no), Some(command)) = (
                                XReadEntryId::from_str(k),
                                AttachBot::from(&s.1.clone()).ok(),
                            ) {
                                stream_data.insert(seq_no, StreamData::AB(command));
                            } else {
                                warn!("fail attach bot xread 0")
                            }
                        } else {
                            warn!("Fail attach bot XREAD 1")
                        }
                    }
                }
            } else {
                warn!("Ignoring topic {}", &xread_topic[..])
            }
        }
    }

    stream_data
}
