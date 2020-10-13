use super::stream::StreamData;
use super::{AllEntryIds, RedisPool};
use crate::topics;
use log::{error, warn};
use r2d2_redis::redis;
use redis_streams::XReadEntryId;
use serde::Deserialize;
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
            .arg(topics::BOT_ATTACHED_TOPIC)
            .arg(topics::MOVE_MADE_TOPIC)
            .arg(entry_ids.bot_attached_xid.to_string())
            .arg(entry_ids.move_made_xid.to_string())
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
    let mut stream_data: HashMap<XReadEntryId, StreamData> = HashMap::new();

    for hash in xread_result.iter() {
        for (xread_topic, xread_data) in hash.iter() {
            if &xread_topic[..] == topics::BOT_ATTACHED_TOPIC {
                for with_timestamps in xread_data {
                    for (k, v) in with_timestamps {
                        let shape: Result<(String, Option<Vec<u8>>), _> = // data <bin> 
                            redis::FromRedisValue::from_redis_value(&v);
                        if let Ok(s) = shape {
                            if let (Ok(xid), Some(bot_attached)) = (
                                XReadEntryId::from_str(k),
                                s.1.clone().and_then(|bytes| {
                                    micro_model_bot::gateway::BotAttached::from(&bytes).ok()
                                }),
                            ) {
                                stream_data.insert(xid, StreamData::BotAttached(bot_attached));
                            } else {
                                warn!("Xread: Deser error   bot attached data")
                            }
                        } else {
                            error!("Fail XREAD bot attached")
                        }
                    }
                }
            } else if &xread_topic[..] == topics::MOVE_MADE_TOPIC {
                for with_timestamps in xread_data {
                    for (k, v) in with_timestamps {
                        let shape: Result<(String, String, String, Vec<u8>), _> = // game_id <uuid-str> data <bin>
                            redis::FromRedisValue::from_redis_value(&v);
                        if let Ok(s) = shape {
                            if let (Ok(xid), Some(_game_id), Some(move_made)) = (
                                XReadEntryId::from_str(k),
                                Uuid::from_str(&s.1).ok(),
                                bincode::deserialize::<move_model::MoveMade>(&s.3.clone()).ok(),
                            ) {
                                stream_data.insert(xid, StreamData::MoveMade(move_made));
                            } else {
                                error!("fail  move made xread inner")
                            }
                        } else {
                            error!("Fail move made XREAD outer")
                        }
                    }
                }
            } else if &xread_topic[..] == topics::HISTORY_PROVIDED_TOPIC {
                for with_timestamps in xread_data {
                    for (k, v) in with_timestamps {
                        let shape: Result<(String, Option<Vec<u8>>), _> = // data <bin> 
                            redis::FromRedisValue::from_redis_value(&v);
                        if let Ok(s) = shape {
                            if let (Ok(xid), Some(hist_prov)) = (
                                XReadEntryId::from_str(k),
                                bincode::deserialize::<sync_model::api::HistoryProvided>(
                                    &s.1.unwrap_or_default(),
                                )
                                .ok(),
                            ) {
                                stream_data.insert(xid, StreamData::HistoryProvided(hist_prov));
                            } else {
                                warn!("Xread: Deser error   hist prov data")
                            }
                        } else {
                            error!("Fail XREAD hist prov")
                        }
                    }
                }
            } else if &xread_topic[..] == topics::SYNC_REPLY_TOPIC {
                for with_timestamps in xread_data {
                    for (k, v) in with_timestamps {
                        let shape: Result<(String, Option<Vec<u8>>), _> = // data <bin> 
                            redis::FromRedisValue::from_redis_value(&v);
                        if let Ok(s) = shape {
                            if let (Ok(xid), Some(sync_reply)) = (
                                XReadEntryId::from_str(k),
                                bincode::deserialize::<sync_model::api::SyncReply>(
                                    &s.1.unwrap_or_default(),
                                )
                                .ok(),
                            ) {
                                stream_data.insert(xid, StreamData::SyncReply(sync_reply));
                            } else {
                                warn!("Xread: Deser error  sync reply data")
                            }
                        } else {
                            error!("Fail XREAD sync reply")
                        }
                    }
                }
            } else if &xread_topic[..] == topics::WAIT_FOR_OPPONENT_TOPIC {
                todo!()
            } else {
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO
                // TODO// TODO// TODO// TODO// TODO// TODO

                warn!("Ignoring topic {}", &xread_topic[..])
            }
        }
    }

    stream_data
}

struct DeserErr;
fn bin_data_process<'a, T>(
    xread_data: &Vec<HashMap<String, redis::Value, StreamData>>,
    mut stream_data: HashMap<XReadEntryId, StreamData>,
    des: Box<dyn Fn(Vec<u8>) -> Result<T, DeserErr>>,
) where
    T: Deserialize<'a> + std::convert::Into<crate::redis_io::stream::StreamData>,
{
    for with_timestamps in xread_data {
        for (k, v) in with_timestamps {
            let shape: Result<(String, Option<Vec<u8>>), _> = // data <bin> 
                redis::FromRedisValue::from_redis_value(&v);
            if let Ok(s) = shape {
                if let (Ok(xid), Some(local)) =
                    (XReadEntryId::from_str(k), des(s.1.unwrap_or_default()).ok())
                {
                    stream_data.insert(xid, local.into());
                } else {
                    warn!("Xread: Deser error  sync reply data")
                }
            } else {
                error!("Fail XREAD sync reply")
            }
        }
    }
}
impl From<sync_model::api::SyncReply> for StreamData {
    fn from(h: sync_model::api::SyncReply) -> Self {
        StreamData::SyncReply(h)
    }
}
