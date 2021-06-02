use super::*;
use log::error;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Client, Commands};
use redis_streams::XId;
use std::collections::HashMap;
use std::rc::Rc;
use redis_streams::anyhow;

  
 
const CONSUMER_NAME: &str = "singleton";
 
fn deser(srr: StreamReadReply) -> Result<HashMap<XId, StreamInput>, StreamDeserErr> {
    let mut out = HashMap::new();
    for k in srr.keys {
        let key = k.key;
        for e in k.ids {
            if let Ok(eid) = XId::from_str(&e.id) {
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

fn ack(client: &Client, key: &str, ids: &[XId]) -> Result<(), StreamAckErr> {
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
