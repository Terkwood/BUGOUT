use super::stream::StreamData;
use super::{AllEntryIds, RedisPool};
use crate::topics::{BOT_ATTACHED_TOPIC, MOVE_MADE_TOPIC};
use redis_streams::XReadEntryId;

use log::{error, warn};
use r2d2_redis::redis;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// performs a redis xread then sorts the results
///
/// entry_ids: the minimum entry ids from which to read
pub trait XReader {
    fn xread_sorted(
        &self,
        entry_ids: super::AllEntryIds,
    ) -> Result<Vec<(XReadEntryId, StreamData)>, redis::RedisError>;
}

const BLOCK_MSEC: u32 = 5000;

pub type XReadResult = Vec<HashMap<String, Vec<HashMap<String, redis::Value>>>>;

pub struct RedisXReader {
    pub pool: Arc<RedisPool>,
}
impl XReader for RedisXReader {
    fn xread_sorted(
        &self,
        entry_ids: AllEntryIds,
    ) -> Result<std::vec::Vec<(XReadEntryId, StreamData)>, redis::RedisError> {
        let mut conn = self.pool.get().unwrap();
        let xrr = redis::cmd("XREAD")
            .arg("BLOCK")
            .arg(&BLOCK_MSEC.to_string())
            .arg("STREAMS")
            .arg(BOT_ATTACHED_TOPIC)
            .arg(MOVE_MADE_TOPIC)
            .arg(entry_ids.bot_attached_eid.to_string())
            .arg(entry_ids.move_made_eid.to_string())
            .query::<XReadResult>(&mut *conn)?;
        let unsorted: HashMap<XReadEntryId, StreamData> = deser(xrr);
        let sorted_keys: Vec<XReadEntryId> = {
            let mut ks: Vec<XReadEntryId> = unsorted.keys().map(|k| *k).collect();
            ks.sort();
            ks
        };
        let mut answer: Vec<(XReadEntryId, StreamData)> = vec![];
        for sk in sorted_keys {
            if let Some(data) = unsorted.get(&sk) {
                answer.push((sk, data.clone()))
            } else {
                error!("💫 {:?}", sk)
            }
        }
        Ok(answer)
    }
}

fn deser(xread_result: XReadResult) -> HashMap<XReadEntryId, StreamData> {
    let mut stream_data = HashMap::new();

    for hash in xread_result.iter() {
        for (xread_topic, xread_data) in hash.iter() {
            if &xread_topic[..] == BOT_ATTACHED_TOPIC {
                for with_timestamps in xread_data {
                    for (k, v) in with_timestamps {
                        let shape: Result<(String, Option<Vec<u8>>), _> = // data <bin> 
                            redis::FromRedisValue::from_redis_value(&v);
                        if let Ok(s) = shape {
                            if let (Ok(seq_no), Some(bot_attached)) = (
                                XReadEntryId::from_str(k),
                                s.1.clone().and_then(|bytes| {
                                    micro_model_bot::gateway::BotAttached::from(&bytes).ok()
                                }),
                            ) {
                                stream_data.insert(seq_no, StreamData::BotAttached(bot_attached));
                            } else {
                                warn!("Xread: Deser error   bot attached data")
                            }
                        } else {
                            error!("Fail XREAD bot attached")
                        }
                    }
                }
            } else if &xread_topic[..] == MOVE_MADE_TOPIC {
                for with_timestamps in xread_data {
                    for (k, v) in with_timestamps {
                        let shape: Result<(String, String, String, Vec<u8>), _> = // game_id <uuid-str> data <bin>
                            redis::FromRedisValue::from_redis_value(&v);
                        if let Ok(s) = shape {
                            if let (Ok(seq_no), Some(_game_id), Some(move_made)) = (
                                XReadEntryId::from_str(k),
                                Uuid::from_str(&s.1).ok(),
                                bincode::deserialize::<micro_model_moves::MoveMade>(&s.3.clone())
                                    .ok(),
                            ) {
                                stream_data.insert(seq_no, StreamData::MoveMade(move_made));
                            } else {
                                error!("fail  move made xread inner")
                            }
                        } else {
                            error!("Fail move made XREAD outer")
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
