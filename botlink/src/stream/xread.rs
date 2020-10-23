use super::topics;
use log::{trace, warn};
use micro_model_bot::gateway::AttachBot;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Client, Commands};
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::sync::Arc;

const BLOCK_MS: usize = 5000;

pub type XReadResult = Vec<HashMap<String, Vec<HashMap<String, redis::Value>>>>;

/// xread_sorted performs a redis xread then sorts the results
///
/// entry_ids: the minimum entry ids from which to read
pub trait XReader: Send + Sync {
    fn xread_sorted(&self) -> Result<Vec<(XReadEntryId, StreamData)>, redis::RedisError>;
}
pub enum StreamReadError {
    Redis(redis::RedisError),
    Deser,
}
pub struct RedisXReader {
    pub client: Arc<Client>,
}
const GROUP_NAME: &str = "botlink";
const CONSUMER_NAME: &str = "singleton";
impl XReader for RedisXReader {
    fn xread_sorted(&self) -> Result<std::vec::Vec<(XReadEntryId, StreamData)>, redis::RedisError> {
        trace!(
            "xreading from {} and {}",
            topics::ATTACH_BOT_CMD,
            topics::GAME_STATES_CHANGELOG
        );
        match self.client.get_connection() {
            Err(e) => Err(e),
            Ok(mut conn) => {
                let opts = StreamReadOptions::default()
                    .block(BLOCK_MS)
                    .group(GROUP_NAME, CONSUMER_NAME);
                let ser = conn.xread_options(
                    &[topics::ATTACH_BOT_CMD, topics::GAME_STATES_CHANGELOG],
                    &[">", ">"],
                    opts,
                )?;

                match deser(ser) {
                    Ok(unsorted) => {
                        todo!() /*
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
                                    Ok(answer)*/
                    }
                    Err(_) => todo!(),
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum StreamData {
    AB(AttachBot),
    GS(move_model::GameState),
}

fn deser(xread_result: XReadResult) -> Result<HashMap<XReadEntryId, StreamData>, StreamReadError> {
    let mut stream_data = HashMap::new();
    todo!();
    /*
    for hash in xread_result.iter() {
        for (xread_topic, xread_data) in hash.iter() {
            if &xread_topic[..] == topics::GAME_STATES_CHANGELOG {
                for with_timestamps in xread_data {
                    for (k, v) in with_timestamps {
                        let shape: Result<(String, Option<Vec<u8>>), _> = // data <bin>
                            redis::FromRedisValue::from_redis_value(&v);
                        if let Ok(s) = shape {
                            if let (Ok(seq_no), Some(game_state)) = (
                                XReadEntryId::from_str(k),
                                s.1.clone()
                                    .and_then(|bytes| move_model::GameState::from(&bytes).ok()),
                            ) {
                                stream_data.insert(seq_no, StreamData::GS(game_state));
                            } else {
                                warn!("Xread: Deser error around game states data")
                            }
                        } else {
                            warn!("Fail XREAD {:?}", &v)
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
        }*/

    Ok(stream_data)
}
