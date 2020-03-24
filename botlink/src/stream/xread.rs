use super::topics::Topics;
use crate::repo::AllEntryIds;
use log::warn;
use micro_model_bot::gateway::AttachBot;
use micro_model_moves::{GameId, GameState};
use redis_conn_pool::redis;
use redis_conn_pool::Pool;
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

const BLOCK_MSEC: u32 = 5000;

pub type XReadResult = Vec<HashMap<String, Vec<HashMap<String, redis::Value>>>>;

/// xread_sorted performs a redis xread then sorts the results
///
/// entry_ids: the minimum entry ids from which to read
pub trait XReader {
    fn xread_sorted(
        &self,
        entry_ids: AllEntryIds,
        topics: &Topics,
    ) -> Result<Vec<(XReadEntryId, StreamData)>, redis::RedisError>;
}

pub struct RedisXReader {
    pub pool: Pool,
}
impl XReader for RedisXReader {
    fn xread_sorted(
        &self,
        entry_ids: AllEntryIds,
        topics: &Topics,
    ) -> Result<std::vec::Vec<(XReadEntryId, StreamData)>, redis::RedisError> {
        let mut conn = self.pool.get().unwrap();
        let xrr = redis::cmd("XREAD")
            .arg("BLOCK")
            .arg(&BLOCK_MSEC.to_string())
            .arg("STREAMS")
            .arg(&topics.attach_bot_ev)
            .arg(&topics.game_states_changelog)
            .arg(entry_ids.attach_bot_eid.to_string())
            .arg(entry_ids.game_states_eid.to_string())
            .query::<XReadResult>(&mut *conn)?;
        let unsorted = deser(xrr, &topics);
        let sorted_keys: Vec<XReadEntryId> = {
            let mut ks: Vec<XReadEntryId> = unsorted.keys().map(|k| *k).collect();
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

fn deser(xread_result: XReadResult, topics: &Topics) -> HashMap<XReadEntryId, StreamData> {
    let mut stream_data = HashMap::new();

    for hash in xread_result.iter() {
        for (xread_topic, xread_data) in hash.iter() {
            if xread_topic[..] == topics.game_states_changelog {
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
            } else if xread_topic[..] == topics.attach_bot_ev {
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
