use super::GROUP_NAME;
use crate::topics::*;
use lobby_model::api::*;
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
    Other,
}
const READ_OP: &str = ">";

impl XRead for Rc<Client> {
    fn xread_sorted(&self) -> Result<std::vec::Vec<(XReadEntryId, StreamInput)>, XReadErr> {
        if let Ok(mut conn) = self.get_connection() {
            let opts = StreamReadOptions::default()
                .block(BLOCK_MS)
                .group(GROUP_NAME, "singleton");
            let xrr = conn.xread_options(
                &[
                    FIND_PUBLIC_GAME,
                    CREATE_GAME,
                    JOIN_PRIVATE_GAME,
                    SESSION_DISCONNECTED,
                ],
                &[READ_OP, READ_OP, READ_OP, READ_OP],
                opts,
            );

            if let Ok(x) = xrr {
                match deser(x) {
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
                    Err(e) => Err(XReadErr::Deser(e)),
                }
            } else {
                Err(XReadErr::Other)
            }
        } else {
            Err(XReadErr::Other)
        }
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
                    let sd: Option<StreamInput> = if key == FIND_PUBLIC_GAME {
                        bincode::deserialize(&data)
                            .map(|fpg| StreamInput::FPG(fpg))
                            .ok()
                    } else if key == CREATE_GAME {
                        bincode::deserialize(&data)
                            .map(|cg| StreamInput::CG(cg))
                            .ok()
                    } else if key == JOIN_PRIVATE_GAME {
                        bincode::deserialize(&data)
                            .map(|jpg| StreamInput::JPG(jpg))
                            .ok()
                    } else if key == SESSION_DISCONNECTED {
                        bincode::deserialize(&data)
                            .map(|sd| StreamInput::SD(sd))
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
pub enum StreamInput {
    FPG(FindPublicGame),
    CG(CreateGame),
    JPG(JoinPrivateGame),
    SD(SessionDisconnected),
}
