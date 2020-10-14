use crate::topics::*;
use lobby_model::api::*;
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
const MAP_KEY: &str = "data";
const MAX_LEN: usize = 1000;
impl XAdd for Rc<Client> {
    fn xadd(&self, data: StreamOutput) -> Result<(), XAddErr> {
        let (key, bytes_result) = match data {
            StreamOutput::GR(gr) => (GAME_READY, bincode::serialize(&gr)),
            StreamOutput::PGR(p) => (PRIVATE_GAME_REJECTED, bincode::serialize(&p)),
            StreamOutput::WFO(w) => (WAIT_FOR_OPPONENT, bincode::serialize(&w)),
        };

        if let Ok(bytes) = bytes_result {
            let mut m: BTreeMap<&str, &[u8]> = BTreeMap::new();
            m.insert(MAP_KEY, &bytes);
            if let Ok(mut conn) = self.get_connection() {
                conn.xadd_maxlen_map(key, StreamMaxlen::Approx(MAX_LEN), AUTO_ID, m)
                    .map_err(|e| XAddErr::Redis(e))
            } else {
                Err(XAddErr::Conn)
            }
        } else {
            Err(XAddErr::Ser)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamOutput {
    WFO(WaitForOpponent),
    GR(GameReady),
    PGR(PrivateGameRejected),
}
