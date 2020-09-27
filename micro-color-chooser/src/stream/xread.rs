use super::*;
use log::error;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Client, Commands};
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::rc::Rc;

/// Read sorted entries from choose-color-pref and game-ready streams streams and acknowledge them.
pub trait XRead {
    /// Read sorted entries from choose-color-pref and game-ready streams
    fn sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, StreamReadErr>;
    /// Ack entries in choose-color-pref
    fn ack_choose_color_pref(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    /// Ack entries in game-ready
    fn ack_game_ready(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
}

#[derive(Debug)]
pub enum StreamReadErr {
    Deser(StreamDeserErr),
    XRead(redis::RedisError),
    Conn,
}
#[derive(Debug)]
pub enum StreamDeserErr {
    XIDFormat,
    DataDeser,
}
#[derive(Debug)]
pub struct StreamAckErr;

const BLOCK_MS: usize = 5000;
const CONSUMER_NAME: &str = "singleton";
impl XRead for Rc<Client> {
    fn sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, StreamReadErr> {
        if let Ok(mut conn) = self.get_connection() {
            let opts = StreamReadOptions::default()
                .block(BLOCK_MS)
                .group(GROUP_NAME, CONSUMER_NAME);
            let ser = conn.xread_options(
                &[topics::CHOOSE_COLOR_PREF, topics::GAME_READY],
                &[">", ">"],
                opts,
            )?;
            match deser(ser) {
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
                Err(e) => Err(StreamReadErr::Deser(e)),
            }
        } else {
            Err(StreamReadErr::Conn)
        }
    }

    fn ack_choose_color_pref(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::CHOOSE_COLOR_PREF, ids)
    }

    fn ack_game_ready(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(self, topics::GAME_READY, ids)
    }
}

fn deser(srr: StreamReadReply) -> Result<HashMap<XReadEntryId, StreamInput>, StreamDeserErr> {
    let mut out = HashMap::new();
    for k in srr.keys {
        let key = k.key;
        for e in k.ids {
            if let Ok(eid) = XReadEntryId::from_str(&e.id) {
                let maybe_data: Option<Vec<u8>> = e.get("data");
                if let Some(data) = maybe_data {
                    let sd: Option<StreamInput> = if key == topics::CHOOSE_COLOR_PREF {
                        bincode::deserialize(&data)
                            .map(|ccp| StreamInput::CCP(ccp))
                            .ok()
                    } else if key == topics::GAME_READY {
                        bincode::deserialize(&data)
                            .map(|gr| StreamInput::GR(gr))
                            .ok()
                    } else {
                        return Err(StreamDeserErr::DataDeser);
                    };
                    if let Some(s) = sd {
                        out.insert(eid, s);
                    } else {
                        return Err(StreamDeserErr::DataDeser);
                    }
                }
            } else {
                error!("xid format");
                return Err(StreamDeserErr::XIDFormat);
            }
        }
    }
    Ok(out)
}

fn ack(client: &Client, key: &str, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
    let c = client.get_connection();
    if let Ok(mut conn) = c {
        let idstrs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let _: usize = conn.xack(key, GROUP_NAME, &idstrs)?;
        Ok(())
    } else {
        Err(StreamAckErr)
    }
}

impl From<redis::RedisError> for StreamAckErr {
    fn from(_: redis::RedisError) -> Self {
        Self
    }
}

impl From<redis::RedisError> for StreamReadErr {
    fn from(e: redis::RedisError) -> Self {
        StreamReadErr::XRead(e)
    }
}
