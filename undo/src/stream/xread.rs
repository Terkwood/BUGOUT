use super::topics::*;
use super::StreamInput;
use log::{error, warn};
use redis::streams::StreamReadReply;
use redis::Commands;
use std::collections::HashMap;

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

fn _deser(srr: StreamReadReply) -> Result<(), XReadDeserErr> {
    for k in srr.keys {
        let key = k.key;
        for e in k.ids {
            let maybe_data: Option<Vec<u8>> = e.get("data");
            if let Some(data) = maybe_data {
                let _: Option<StreamInput> = if key == GAME_STATES_CHANGELOG {
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
            }
        }
    }
    Ok(())
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
