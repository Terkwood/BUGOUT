use super::stream::StreamData;
use super::{AllEntryIds, RedisPool};
use crate::topics;
use log::{error, warn};
use r2d2_redis::redis;
use redis_streams::XReadEntryId;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

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
            .arg(topics::HISTORY_PROVIDED_TOPIC)
            .arg(topics::SYNC_REPLY_TOPIC)
            .arg(topics::WAIT_FOR_OPPONENT_TOPIC)
            .arg(topics::GAME_READY_TOPIC)
            .arg(topics::PRIVATE_GAME_REJECTED_TOPIC)
            .arg(topics::COLORS_CHOSEN_TOPIC)
            .arg(entry_ids.bot_attached_xid.to_string())
            .arg(entry_ids.move_made_xid.to_string())
            .arg(entry_ids.hist_prov_xid.to_string())
            .arg(entry_ids.sync_reply_xid.to_string())
            .arg(entry_ids.wait_opponent_xid.to_string())
            .arg(entry_ids.game_ready_xid.to_string())
            .arg(entry_ids.priv_game_reject_xid.to_string())
            .arg(entry_ids.colors_chosen_xid.to_string())
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
                        let shape: Result<(String, Vec<u8>), _> = // game_id <uuid-str> data <bin>
                            redis::FromRedisValue::from_redis_value(&v);
                        if let Ok(s) = shape {
                            if let (Ok(xid), Some(move_made)) = (
                                XReadEntryId::from_str(k),
                                bincode::deserialize::<move_model::MoveMade>(&s.1.clone()).ok(),
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
                bin_data_process(
                    xread_data,
                    &mut stream_data,
                    Box::new(|bytes| {
                        bincode::deserialize::<sync_model::api::HistoryProvided>(&bytes)
                    }),
                    topics::HISTORY_PROVIDED_TOPIC,
                )
            } else if &xread_topic[..] == topics::SYNC_REPLY_TOPIC {
                bin_data_process(
                    xread_data,
                    &mut stream_data,
                    Box::new(|bytes| bincode::deserialize::<sync_model::api::SyncReply>(&bytes)),
                    topics::SYNC_REPLY_TOPIC,
                )
            } else if &xread_topic[..] == topics::WAIT_FOR_OPPONENT_TOPIC {
                bin_data_process(
                    xread_data,
                    &mut stream_data,
                    Box::new(|bytes| {
                        bincode::deserialize::<lobby_model::api::WaitForOpponent>(&bytes)
                    }),
                    topics::WAIT_FOR_OPPONENT_TOPIC,
                )
            } else if &xread_topic[..] == topics::GAME_READY_TOPIC {
                bin_data_process(
                    xread_data,
                    &mut stream_data,
                    Box::new(|bytes| bincode::deserialize::<lobby_model::api::GameReady>(&bytes)),
                    topics::GAME_READY_TOPIC,
                )
            } else if &xread_topic[..] == topics::PRIVATE_GAME_REJECTED_TOPIC {
                bin_data_process(
                    xread_data,
                    &mut stream_data,
                    Box::new(|bytes| {
                        bincode::deserialize::<lobby_model::api::PrivateGameRejected>(&bytes)
                    }),
                    topics::PRIVATE_GAME_REJECTED_TOPIC,
                )
            } else if &xread_topic[..] == topics::COLORS_CHOSEN_TOPIC {
                bin_data_process(
                    xread_data,
                    &mut stream_data,
                    Box::new(|bytes| {
                        bincode::deserialize::<color_model::api::ColorsChosen>(&bytes)
                    }),
                    topics::COLORS_CHOSEN_TOPIC,
                )
            } else {
                warn!("Ignoring topic {}", &xread_topic[..])
            }
        }
    }

    stream_data
}

fn bin_data_process<'a, T>(
    xread_data: &Vec<HashMap<String, redis::Value, std::collections::hash_map::RandomState>>,
    stream_data: &mut HashMap<XReadEntryId, StreamData>,
    des: Box<dyn Fn(Vec<u8>) -> Result<T, Box<bincode::ErrorKind>>>,
    topic: &str,
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
                    warn!("Xread: Deser error  {}", topic)
                }
            } else {
                error!("Fail XREAD {}", topic)
            }
        }
    }
}
impl From<sync_model::api::HistoryProvided> for StreamData {
    fn from(a: sync_model::api::HistoryProvided) -> Self {
        StreamData::HistoryProvided(a)
    }
}
impl From<sync_model::api::SyncReply> for StreamData {
    fn from(h: sync_model::api::SyncReply) -> Self {
        StreamData::SyncReply(h)
    }
}
impl From<lobby_model::api::WaitForOpponent> for StreamData {
    fn from(w: lobby_model::api::WaitForOpponent) -> Self {
        StreamData::WaitForOpponent(w)
    }
}
impl From<lobby_model::api::GameReady> for StreamData {
    fn from(w: lobby_model::api::GameReady) -> Self {
        StreamData::GameReady(w)
    }
}
impl From<lobby_model::api::PrivateGameRejected> for StreamData {
    fn from(w: lobby_model::api::PrivateGameRejected) -> Self {
        StreamData::PrivGameRejected(w)
    }
}
impl From<color_model::api::ColorsChosen> for StreamData {
    fn from(w: color_model::api::ColorsChosen) -> Self {
        StreamData::ColorsChosen(w)
    }
}
