use super::{topics, GROUP_NAME};
use crate::stream::StreamInput;
use core_model::GameId;
use log::error;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Client, Commands};
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;

const INIT_ACK_CAPACITY: usize = 25;
const BLOCK_MS: usize = 5000;
const CONSUMER_NAME: &str = "singleton";

pub trait XRead {
    fn read_sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, StreamReadErr>;
    fn ack_req_sync(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_prov_hist(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_game_states(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn ack_move_made(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
}

impl XRead for Rc<Client> {
    fn read_sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, StreamReadErr> {
        if let Ok(mut conn) = self.get_connection() {
            let opts = StreamReadOptions::default()
                .block(BLOCK_MS)
                .group(GROUP_NAME, CONSUMER_NAME);
            let ser = conn.xread_options(
                &[topics::GAME_STATES_CHANGELOG, topics::PROVIDE_HISTORY],
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

    fn ack_req_sync(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(&self, topics::REQ_SYNC, ids)
    }
    fn ack_prov_hist(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(&self, topics::PROVIDE_HISTORY, ids)
    }
    fn ack_game_states(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(&self, topics::GAME_STATES_CHANGELOG, ids)
    }
    fn ack_move_made(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        ack(&self, topics::MOVE_MADE, ids)
    }
}
#[derive(Debug)]
pub enum StreamReadErr {
    Deser(StreamDeserErr),
    XRead(redis::RedisError),
    Conn,
}
#[derive(Debug)]
pub enum StreamDeserErr {
    EIDFormat,
    DataDeser,
}
#[derive(Debug)]
pub struct StreamAckErr;

pub struct Unacknowledged {
    req_sync: Vec<XReadEntryId>,
    prov_hist: Vec<XReadEntryId>,
    game_states: Vec<XReadEntryId>,
    move_made: Vec<XReadEntryId>,
}

impl Unacknowledged {
    pub fn ack_all(&mut self, components: &crate::Components) {
        if !self.req_sync.is_empty() {
            if let Err(_e) = components.xread.ack_req_sync(&self.req_sync) {
                error!("ack for req sync failed")
            } else {
                self.req_sync.clear();
            }
        }

        if !self.prov_hist.is_empty() {
            if let Err(_e) = components.xread.ack_prov_hist(&self.prov_hist) {
                error!("ack for provide history failed")
            } else {
                self.prov_hist.clear();
            }
        }
        if !self.game_states.is_empty() {
            if let Err(_e) = components.xread.ack_game_states(&self.game_states) {
                error!("ack for game states failed")
            } else {
                self.game_states.clear();
            }
        }
        if !self.move_made.is_empty() {
            if let Err(_e) = components.xread.ack_move_made(&self.move_made) {
                error!("ack for move made failed")
            } else {
                self.move_made.clear();
            }
        }
    }
    pub fn push(&mut self, xid: XReadEntryId, event: StreamInput) {
        match event {
            StreamInput::GS(_, _) => self.game_states.push(xid),
            StreamInput::MM(_) => self.move_made.push(xid),
            StreamInput::PH(_) => self.prov_hist.push(xid),
            StreamInput::RS(_) => self.req_sync.push(xid),
        }
    }
}

impl Default for Unacknowledged {
    fn default() -> Self {
        Self {
            prov_hist: Vec::with_capacity(INIT_ACK_CAPACITY),
            req_sync: Vec::with_capacity(INIT_ACK_CAPACITY),
            game_states: Vec::with_capacity(INIT_ACK_CAPACITY),
            move_made: Vec::with_capacity(INIT_ACK_CAPACITY),
        }
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
                    } else if key == topics::REQ_SYNC {
                        bincode::deserialize(&data)
                            .map(|rs| StreamInput::RS(rs))
                            .ok()
                    } else if key == topics::PROVIDE_HISTORY {
                        bincode::deserialize(&data)
                            .map(|ph| StreamInput::PH(ph))
                            .ok()
                    } else if key == topics::MOVE_MADE {
                        bincode::deserialize(&data)
                            .map(|mm| StreamInput::MM(mm))
                            .ok()
                    } else {
                        error!("Unknown key {}", key);
                        return Err(StreamDeserErr::DataDeser);
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
        StreamReadErr::XRead(e)
    }
}
