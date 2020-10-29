use super::topics;
use super::StreamInput;
use log::{error, trace};
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
    fn xread_sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, StreamReadError>;
}

#[derive(Debug)]
pub enum StreamReadError {
    Redis(redis::RedisError),
    Deser,
}

const CONSUMER_NAME: &str = "singleton";
impl XReader for Arc<Client> {
    fn xread_sorted(&self) -> Result<std::vec::Vec<(XReadEntryId, StreamInput)>, StreamReadError> {
        trace!(
            "xreading from {} and {}",
            topics::ATTACH_BOT_CMD,
            topics::GAME_STATES_CHANGELOG
        );
        match self.get_connection() {
            Err(e) => Err(StreamReadError::Redis(e)),
            Ok(mut conn) => {
                let opts = StreamReadOptions::default()
                    .block(BLOCK_MS)
                    .group(super::GROUP_NAME, CONSUMER_NAME);
                let ser = conn.xread_options(
                    &[topics::ATTACH_BOT_CMD, topics::GAME_STATES_CHANGELOG],
                    &[">", ">"],
                    opts,
                )?;

                match deser(ser) {
                    Ok(unsorted) => {
                        let mut sorted_keys: Vec<XReadEntryId> =
                            unsorted.keys().map(|k| *k).collect();
                        sorted_keys.sort();

                        let mut answer = vec![];
                        for sk in sorted_keys {
                            if let Some(data) = unsorted.get(&sk) {
                                answer.push((sk, data.clone()))
                            }
                        }
                        Ok(answer)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
}

fn deser(xrr: StreamReadReply) -> Result<HashMap<XReadEntryId, StreamInput>, StreamReadError> {
    let mut out = HashMap::new();
    for k in xrr.keys {
        let key = k.key;
        for e in k.ids {
            if let Ok(xid) = XReadEntryId::from_str(&e.id) {
                let maybe_data: Option<Vec<u8>> = e.get("data");
                if let Some(data) = maybe_data {
                    let sd: Option<StreamInput> = if key == topics::GAME_STATES_CHANGELOG {
                        bincode::deserialize(&data)
                            .map(|gs| StreamInput::GS(gs))
                            .ok()
                    } else if key == topics::ATTACH_BOT_CMD {
                        bincode::deserialize(&data)
                            .map(|ab| StreamInput::AB(ab))
                            .ok()
                    } else {
                        error!("Unknown key {}", key);
                        return Err(StreamReadError::Deser);
                    };
                    if let Some(s) = sd {
                        out.insert(xid, s);
                    } else {
                        error!("empty data");
                        return Err(StreamReadError::Deser);
                    }
                } else {
                    return Err(StreamReadError::Deser);
                }
            }
        }
    }
    Ok(out)
}

impl From<redis::RedisError> for StreamReadError {
    fn from(e: redis::RedisError) -> Self {
        StreamReadError::Redis(e)
    }
}
