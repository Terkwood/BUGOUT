use super::*;
use log::error;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Client, Commands};
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;

pub trait XRead {
    fn sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, StreamReadErr>;
    fn ack_choose_color_pref(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
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
    EIDFormat,
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
                Ok(unsorted) => todo!(),
                Err(_) => todo!(),
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
    todo!()
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
