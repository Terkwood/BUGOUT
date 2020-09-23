use super::{topics, GROUP_NAME};
use crate::model::GameId;
use crate::stream::StreamInput;
use log::error;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Client, Commands};
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;

pub trait XRead {
    fn xread_sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, StreamReadErr>;
    fn xack_prov_hist(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn xack_game_states(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
}

#[derive(Debug)]
pub enum StreamReadErr {
    Deser(StreamDeserErr),
    Other,
}
#[derive(Debug)]
pub enum StreamDeserErr {
    EIDFormat,
    DataDeser,
}
#[derive(Debug)]
pub struct StreamAckErr;

const BLOCK_MS: usize = 5000;
const CONSUMER_NAME: &str = "singleton";
impl XRead for Rc<Client> {
    fn xread_sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, StreamReadErr> {
        if let Ok(mut conn) = self.get_connection() {
            let mut conn = self.get_connection().expect("conn");
            let opts = StreamReadOptions::default()
                .block(BLOCK_MS)
                .group(GROUP_NAME, CONSUMER_NAME);
            let ser = conn.xread_options(
                &[topics::GAME_STATES_CHANGELOG, topics::PROVIDE_HISTORY],
                &[">", ">"],
                opts,
            )?;
        }

        todo!()
    }

    fn xack_prov_hist(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(&self, topics::PROVIDE_HISTORY, ids)
    }

    fn xack_game_states(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(&self, topics::GAME_STATES_CHANGELOG, ids)
    }
}

fn ack(client: &Client, key: &str, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
    let mut conn = client.get_connection().expect("conn");
    let idstrs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
    let _: usize = conn.xack(key, GROUP_NAME, &idstrs)?;
    Ok(())
}

fn deser(srr: StreamReadReply) -> Result<HashMap<XReadEntryId, StreamInput>, StreamDeserErr> {
    let mut out = HashMap::new();
    for k in srr.keys {
        let key = k.key;
        for e in k.ids {
            if let Ok(eid) = XReadEntryId::from_str(&e.id) {
                let maybe_data: Option<Vec<u8>> = e.get("data");
                if let Some(data) = maybe_data {
                    let sd: Option<StreamInput> = if key == topics::GAME_STATES_CHANGELOG {
                        let eg: Option<String> = e.get("game_id");
                        let game_id = GameId(
                            eg.as_ref()
                                .map(|u_s| Uuid::parse_str(u_s).ok())
                                .flatten()
                                .unwrap_or(Uuid::nil()),
                        );
                        bincode::deserialize(&data)
                            .map(|gs| StreamInput::GS(game_id, gs))
                            .ok()
                    } else if key == topics::PROVIDE_HISTORY {
                        bincode::deserialize(&data)
                            .map(|ph| StreamInput::PH(ph))
                            .ok()
                    } else {
                        error!("Unknown key {}", key);
                        None
                    };
                    if let Some(s) = sd {
                        out.insert(eid, s);
                    } else {
                        return Err(StreamDeserErr::DataDeser);
                    }
                }
            } else {
                error!("eid-ish");
                return Err(StreamDeserErr::EIDFormat);
            }
        }
    }
    Ok(out)
}

impl From<redis::RedisError> for StreamAckErr {
    fn from(_: redis::RedisError) -> Self {
        Self
    }
}
impl From<redis::RedisError> for StreamReadErr {
    fn from(e: redis::RedisError) -> Self {
        StreamReadErr::Deser(StreamDeserErr::DataDeser)
    }
}
