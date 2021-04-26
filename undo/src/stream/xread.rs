use super::topics::*;
use super::{StreamInput, GROUP_NAME};
use log::{error, warn};
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Client, Commands};
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::rc::Rc;

const BLOCK_MS: usize = 5000;

/// xread_sorted performs a redis xread then sorts the results
///
/// entry_ids: the minimum entry ids from which to read
pub trait XRead {
    fn xread_sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, XReadErr>;
}

#[derive(Debug)]
pub enum XReadErr {
    Deser(XReadDeserErr),
    Redis(redis::RedisError),
    Other,
}

#[derive(Debug)]
pub enum XReadDeserErr {
    XIDFormat,
    DataDeser(String),
}

const READ_OP: &str = ">";
const CONSUMER_NAME: &str = "singleton";

impl XRead for Rc<Client> {
    fn xread_sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, XReadErr> {
        let mut conn = self.get_connection()?;

        let opts = StreamReadOptions::default()
            .block(BLOCK_MS)
            .group(GROUP_NAME, CONSUMER_NAME);
        let xrr = conn.xread_options(
            &[UNDO_MOVE, BOT_ATTACHED, GAME_STATES_CHANGELOG],
            &[READ_OP, READ_OP, READ_OP],
            opts,
        )?;
        let unsorted = deser(xrr)?;
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
}

fn deser(srr: StreamReadReply) -> Result<HashMap<XReadEntryId, StreamInput>, XReadDeserErr> {
    let mut out = HashMap::new();
    for k in srr.keys {
        let key = k.key;
        for e in k.ids {
            if let Ok(eid) = XReadEntryId::from_str(&e.id) {
                let maybe_data: Option<Vec<u8>> = e.get("data");
                if let Some(data) = maybe_data {
                    let sd: Option<StreamInput> = if key == GAME_STATES_CHANGELOG {
                        bincode::deserialize(&data)
                            .map(|gs| StreamInput::LOG(gs))
                            .ok()
                    } else if key == BOT_ATTACHED {
                        bincode::deserialize(&data)
                            .map(|ba| StreamInput::BA(ba))
                            .ok()
                    } else if key == UNDO_MOVE {
                        bincode::deserialize(&data)
                            .map(|um| StreamInput::UM(um))
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
                error!("cannot read stream entry id");
                return Err(XReadDeserErr::XIDFormat);
            }
        }
    }
    Ok(out)
}

impl From<redis::RedisError> for XReadErr {
    fn from(r: redis::RedisError) -> Self {
        XReadErr::Redis(r)
    }
}

impl From<XReadDeserErr> for XReadErr {
    fn from(d: XReadDeserErr) -> Self {
        Self::Deser(d)
    }
}
