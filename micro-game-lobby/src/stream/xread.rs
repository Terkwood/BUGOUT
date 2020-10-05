use crate::repo::AllEntryIds;
use crate::repo::EntryIdType;
use crate::topics::*;
use community_redis_streams::{StreamCommands, StreamReadOptions, StreamReadReply};
use lobby_model::api::*;
use log::{error, warn};
use redis::Client;
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::rc::Rc;

const BLOCK_MSEC: usize = 5000;

/// xread_sorted performs a redis xread then sorts the results
///
/// entry_ids: the minimum entry ids from which to read
pub trait XRead {
    fn xread_sorted(
        &self,
        entry_ids: AllEntryIds,
    ) -> Result<Vec<(XReadEntryId, StreamInput)>, XReadErr>;
}

#[derive(Debug)]
pub enum XReadErr {
    Deser(XReadDeserErr),
    Other,
}
impl XRead for Rc<Client> {
    fn xread_sorted(
        &self,
        entry_ids: AllEntryIds,
    ) -> Result<std::vec::Vec<(XReadEntryId, StreamInput)>, XReadErr> {
        if let Ok(mut conn) = self.get_connection() {
            let opts = StreamReadOptions::default().block(BLOCK_MSEC);
            let xrr: Result<StreamReadReply, _> = conn.xread_options(
                &[FIND_PUBLIC_GAME, CREATE_GAME, JOIN_PRIVATE_GAME],
                &[
                    entry_ids.find_public_game.to_string(),
                    entry_ids.create_game.to_string(),
                    entry_ids.join_private_game.to_string(),
                    entry_ids.session_disconnected.to_string(),
                ],
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

impl From<crate::stream::StreamInput> for EntryIdType {
    fn from(s: crate::stream::StreamInput) -> Self {
        match s {
            StreamInput::FPG(_) => EntryIdType::FindPublicGameCmd,
            StreamInput::CG(_) => EntryIdType::CreateGameCmd,
            StreamInput::JPG(_) => EntryIdType::JoinPrivateGameCmd,
            StreamInput::SD(_) => EntryIdType::SessionDisconnectedEv,
        }
    }
}
