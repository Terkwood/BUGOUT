use super::topics::*;
use super::StreamOutput;
use redis::Client;
use redis::{streams::StreamMaxlen, Commands};
use std::collections::BTreeMap;
use std::rc::Rc;

pub trait XAdd {
    fn xadd(&self, output: &StreamOutput) -> Result<(), StreamAddErr>;
}
#[derive(Debug)]
pub enum StreamAddErr {
    Redis(redis::RedisError),
    Ser,
    Conn,
}

const AUTO_ID: &str = "*";
const DATA_KEY: &str = "data";
const MAX_LEN: usize = 1000;
impl XAdd for Rc<Client> {
    fn xadd(&self, output: &StreamOutput) -> Result<(), StreamAddErr> {
        let (key, bytes_result) = match &output {
            StreamOutput::MU(move_undone) => (MOVE_UNDONE, bincode::serialize(&move_undone)),
            StreamOutput::LOG(state) => (GAME_STATES_CHANGELOG, bincode::serialize(&state)),
            StreamOutput::REJECT(original_undo) => {
                (UNDO_REJECTED, bincode::serialize(&original_undo))
            }
        };
        if let Ok(bytes) = bytes_result {
            let mut m: BTreeMap<&str, &[u8]> = BTreeMap::new();

            m.insert(DATA_KEY, &bytes);

            xadd_io(&self, key, m)
        } else {
            Err(StreamAddErr::Ser)
        }
    }
}

fn xadd_io(client: &Client, key: &str, m: BTreeMap<&str, &[u8]>) -> Result<(), StreamAddErr> {
    if let Ok(mut conn) = client.get_connection() {
        conn.xadd_maxlen_map(key, StreamMaxlen::Approx(MAX_LEN), AUTO_ID, m)
            .map_err(|e| StreamAddErr::Redis(e))
    } else {
        Err(StreamAddErr::Conn)
    }
}
