use super::topics::{CREATE_GAME, FIND_PUBLIC_GAME, JOIN_PRIVATE_GAME};
use crate::api::*;
use crate::repo::AllEntryIds;
use community_redis_streams::{StreamCommands, StreamReadOptions, StreamReadReply};
use log::{error, warn};
use redis_conn_pool::Pool;
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::sync::Arc;

const BLOCK_MSEC: usize = 5000;

/// xread_sorted performs a redis xread then sorts the results
///
/// entry_ids: the minimum entry ids from which to read
pub trait XReader: Send + Sync {
    fn xread_sorted(
        &self,
        entry_ids: AllEntryIds,
    ) -> Result<Vec<(XReadEntryId, StreamData)>, XReadErr>;
}

pub struct RedisXReader {
    pub pool: Arc<Pool>,
}

#[derive(Debug)]
pub enum XReadErr {
    Deser(XReadDeserErr),
    Other,
}
impl XReader for RedisXReader {
    fn xread_sorted(
        &self,
        entry_ids: AllEntryIds,
    ) -> Result<std::vec::Vec<(XReadEntryId, StreamData)>, XReadErr> {
        let mut conn = self.pool.get().expect("Pool");
        let opts = StreamReadOptions::default().block(BLOCK_MSEC);
        let xrr: Result<StreamReadReply, _> = conn.xread_options(
            &[FIND_PUBLIC_GAME, CREATE_GAME, JOIN_PRIVATE_GAME],
            &[
                entry_ids.find_public_game.to_string(),
                entry_ids.create_game.to_string(),
                entry_ids.join_private_game.to_string(),
            ],
            opts,
        );
        if let Ok(x) = xrr {
            match deser(x) {
                Ok(unsorted) => {
                    let mut sorted_keys: Vec<XReadEntryId> = unsorted.keys().map(|k| *k).collect();
                    sorted_keys.sort();

                    let mut answer = vec![];
                    for sk in sorted_keys {
                        if let Some(data) = unsorted.get(&sk) {
                            answer.push((sk, data.clone()))
                        }
                    }
                    Ok(answer)
                }
                Err(e) => Err(XReadErr::Deser(e)),
            }
        } else {
            Err(XReadErr::Other)
        }
    }
}

fn deser(srr: StreamReadReply) -> Result<HashMap<XReadEntryId, StreamData>, XReadDeserErr> {
    let mut out = HashMap::new();
    for k in srr.keys {
        let key = k.key;
        for e in k.ids {
            if let Ok(eid) = XReadEntryId::from_str(&e.id) {
                let maybe_data: Option<Vec<u8>> = e.get("data");
                for data in maybe_data {
                    let sd: Option<StreamData> = if key == FIND_PUBLIC_GAME {
                        bincode::deserialize(&data)
                            .map(|fpg| StreamData::FPG(fpg))
                            .ok()
                    } else if key == CREATE_GAME {
                        bincode::deserialize(&data)
                            .map(|cg| StreamData::CG(cg))
                            .ok()
                    } else if key == JOIN_PRIVATE_GAME {
                        bincode::deserialize(&data)
                            .map(|jpg| StreamData::JPG(jpg))
                            .ok()
                    } else {
                        warn!("Unknown key {}", key);
                        None
                    };
                    if let Some(s) = sd {
                        out.insert(eid, s);
                    } else {
                        return Err(XReadDeserErr::DataDeser(key));
                    }
                }
            } else {
                error!("eid-ish");
                return Err(XReadDeserErr::EIDFormat);
            }
        }
    }
    Ok(out)
}

#[derive(Debug)]
pub enum XReadDeserErr {
    EIDFormat,
    DataDeser(String),
}

#[derive(Clone, Debug)]
pub enum StreamData {
    FPG(FindPublicGame),
    CG(CreateGame),
    JPG(JoinPrivateGame),
}
