use super::StreamOutput;
use crate::topics::*;
use redis::Client;
use redis::{streams::StreamMaxlen, Commands};
use std::collections::BTreeMap;
use std::rc::Rc;

pub trait XAdd {
    fn xadd(&self, data: StreamOutput) -> Result<(), XAddErr>;
}

#[derive(Debug)]
pub enum XAddErr {
    Redis(redis::RedisError),
    Ser,
    Conn,
}

const AUTO_ID: &str = "*";
const DATA_KEY: &str = "data";
const MAX_LEN: usize = 1000;
impl XAdd for Rc<Client> {
    fn xadd(&self, data: StreamOutput) -> Result<(), XAddErr> {
        let (key, bytes_result) = match &data {
            StreamOutput::GR(gr) => (GAME_READY, bincode::serialize(&gr)),
            StreamOutput::PGR(p) => (PRIVATE_GAME_REJECTED, bincode::serialize(&p)),
            StreamOutput::WFO(w) => (WAIT_FOR_OPPONENT, bincode::serialize(&w)),
            StreamOutput::LOG(state) => (GAME_STATES_CHANGELOG, bincode::serialize(&state)),
        };
        if let Ok(bytes) = bytes_result {
            let mut m: BTreeMap<&str, &[u8]> = BTreeMap::new();

            m.insert(DATA_KEY, &bytes);

            xadd_io(&self, key, m)
        } else {
            Err(XAddErr::Ser)
        }
    }
}

fn xadd_io(client: &Client, key: &str, m: BTreeMap<&str, &[u8]>) -> Result<(), XAddErr> {
    if let Ok(mut conn) = client.get_connection() {
        conn.xadd_maxlen_map(key, StreamMaxlen::Approx(MAX_LEN), AUTO_ID, m)
            .map_err(|e| XAddErr::Redis(e))
    } else {
        Err(XAddErr::Conn)
    }
}
