use super::{StreamData, GROUP_NAME};
use crate::topics;
use log::{error, warn};
use redis::{
    streams::{StreamReadOptions, StreamReadReply},
    Commands,
};
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::sync::Arc;

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

#[derive(Debug)]
pub struct StreamDeserErr;

pub type XReadResult = Vec<HashMap<String, Vec<HashMap<String, redis::Value>>>>;

const CONSUMER_NAME: &str = "singleton";
pub fn create_consumer_group(client: &redis::Client) {
    let mut conn = client.get_connection().expect("group create conn");
    let to_create = vec![
        topics::MOVE_MADE_TOPIC,
        topics::HISTORY_PROVIDED_TOPIC,
        topics::SYNC_REPLY_TOPIC,
        topics::WAIT_FOR_OPPONENT_TOPIC,
        topics::GAME_READY_TOPIC,
        topics::PRIVATE_GAME_REJECTED_TOPIC,
        topics::COLORS_CHOSEN_TOPIC,
        topics::BOT_ATTACHED_TOPIC,
    ];
    for topic in to_create {
        let created: Result<(), _> = conn.xgroup_create_mkstream(topic, GROUP_NAME, "$");
        if let Err(e) = created {
            warn!(
                "Ignoring error creating {} consumer group (it probably exists already) {:?}",
                topic, e
            );
        }
    }
}

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

const BLOCK_MS: usize = 5000;
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
        let key: &str = &k.key;
        for e in k.ids {
            if let Ok(xid) = XReadEntryId::from_str(&e.id) {
                let maybe_data: Option<Vec<u8>> = e.get("data");
                if let Some(data) = maybe_data {
                    let sd: Option<StreamData> = match key {
                        topics::BOT_ATTACHED_TOPIC => bincode::deserialize(&data)
                            .map(|b| StreamData::BotAttached(b))
                            .ok(),
                        topics::MOVE_MADE_TOPIC => bincode::deserialize(&data)
                            .map(|m| StreamData::MoveMade(m))
                            .ok(),
                        topics::HISTORY_PROVIDED_TOPIC => bincode::deserialize(&data)
                            .map(|hp| StreamData::HistoryProvided(hp))
                            .ok(),
                        topics::SYNC_REPLY_TOPIC => bincode::deserialize(&data)
                            .map(|synrep| StreamData::SyncReply(synrep))
                            .ok(),
                        topics::WAIT_FOR_OPPONENT_TOPIC => bincode::deserialize(&data)
                            .map(|w| StreamData::WaitForOpponent(w))
                            .ok(),
                        topics::GAME_READY_TOPIC => bincode::deserialize(&data)
                            .map(|g| StreamData::GameReady(g))
                            .ok(),
                        topics::PRIVATE_GAME_REJECTED_TOPIC => bincode::deserialize(&data)
                            .map(|p| StreamData::PrivGameRejected(p))
                            .ok(),
                        topics::COLORS_CHOSEN_TOPIC => bincode::deserialize(&data)
                            .map(|c| StreamData::ColorsChosen(c))
                            .ok(),
                        _ => {
                            error!("Unknown key {}", key);
                            return Err(StreamDeserErr);
                        }
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
