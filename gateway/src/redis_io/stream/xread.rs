use super::{StreamData, GROUP_NAME};
use crate::topics;
use log::error;
use redis::{
    streams::{StreamReadOptions, StreamReadReply},
    Commands,
};
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::sync::Arc;

const CONSUMER_NAME: &str = "singleton";
/// performs a redis xread then sorts the results
///
/// entry_ids: the minimum entry ids from which to read
pub trait XReader {
    fn xread_sorted(&self) -> Result<Vec<(XReadEntryId, StreamData)>, StreamReadError>;
}

#[derive(Debug)]
pub enum StreamReadError {
    Redis(redis::RedisError),
    Deser(StreamDeserErr),
}

const BLOCK_MS: usize = 5000;

pub type XReadResult = Vec<HashMap<String, Vec<HashMap<String, redis::Value>>>>;

pub struct RedisXReader {
    pub client: Arc<redis::Client>,
}

const ALL_TOPICS: &[&str; 8] = &[
    topics::BOT_ATTACHED_TOPIC,
    topics::MOVE_MADE_TOPIC,
    topics::HISTORY_PROVIDED_TOPIC,
    topics::SYNC_REPLY_TOPIC,
    topics::WAIT_FOR_OPPONENT_TOPIC,
    topics::GAME_READY_TOPIC,
    topics::PRIVATE_GAME_REJECTED_TOPIC,
    topics::COLORS_CHOSEN_TOPIC,
];
lazy_static! {
    static ref AUTO_IDS: Vec<&'static str> = ALL_TOPICS.iter().map(|_| ">").collect();
}

impl XReader for RedisXReader {
    fn xread_sorted(&self) -> Result<std::vec::Vec<(XReadEntryId, StreamData)>, StreamReadError> {
        match self.client.get_connection() {
            Ok(mut conn) => {
                let opts = StreamReadOptions::default()
                    .block(BLOCK_MS)
                    .group(GROUP_NAME, CONSUMER_NAME);

                let ser: StreamReadReply = conn.xread_options(ALL_TOPICS, &AUTO_IDS, opts)?;

                let unsorted: HashMap<XReadEntryId, StreamData> = deser(ser)?;
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
            Err(e) => Err(StreamReadError::Redis(e)),
        }
    }
}
fn deser(srr: StreamReadReply) -> Result<HashMap<XReadEntryId, StreamData>, StreamDeserErr> {
    let mut out: HashMap<XReadEntryId, StreamData> = HashMap::new();

    for k in srr.keys {
        let key = k.key;
        for e in k.ids {
            if let Ok(xid) = XReadEntryId::from_str(&e.id) {
                let maybe_data: Option<Vec<u8>> = e.get("data");
                if let Some(data) = maybe_data {
                    let sd: Option<StreamData> = if key == topics::BOT_ATTACHED_TOPIC {
                        todo!("special")
                    } else if key == topics::MOVE_MADE_TOPIC {
                        todo!("really really special")
                    } else if key == topics::HISTORY_PROVIDED_TOPIC {
                        bincode::deserialize(&data)
                            .map(|hp| StreamData::HistoryProvided(hp))
                            .ok()
                    } else if key == topics::SYNC_REPLY_TOPIC {
                        bincode::deserialize(&data)
                            .map(|synrep| StreamData::SyncReply(synrep))
                            .ok()
                    } else if key == topics::WAIT_FOR_OPPONENT_TOPIC {
                        bincode::deserialize(&data)
                            .map(|w| StreamData::WaitForOpponent(w))
                            .ok()
                    } else if key == topics::GAME_READY_TOPIC {
                        bincode::deserialize(&data)
                            .map(|g| StreamData::GameReady(g))
                            .ok()
                    } else if key == topics::PRIVATE_GAME_REJECTED_TOPIC {
                        bincode::deserialize(&data)
                            .map(|p| StreamData::PrivGameRejected(p))
                            .ok()
                    } else if key == topics::COLORS_CHOSEN_TOPIC {
                        bincode::deserialize(&data)
                            .map(|c| StreamData::ColorsChosen(c))
                            .ok()
                    } else {
                        error!("Unknown key {}", key);
                        return Err(StreamDeserErr);
                    };
                    if let Some(s) = sd {
                        out.insert(xid, s);
                    } else {
                        return Err(StreamDeserErr);
                    }
                } else {
                    error!("xid-ish");
                    return Err(StreamDeserErr);
                }
            }
        }
    }
    Ok(out)
}

#[derive(Debug)]
pub struct StreamDeserErr;

impl From<StreamDeserErr> for StreamReadError {
    fn from(e: StreamDeserErr) -> Self {
        StreamReadError::Deser(e)
    }
}
impl From<redis::RedisError> for StreamReadError {
    fn from(e: redis::RedisError) -> Self {
        StreamReadError::Redis(e)
    }
}
