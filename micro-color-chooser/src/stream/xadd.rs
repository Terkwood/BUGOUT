use super::topics;
use color_model::api::ColorsChosen;
use redis::{streams::StreamMaxlen, Client, Commands};
use std::collections::BTreeMap;
use std::rc::Rc;

pub trait XAdd {
    fn xadd(&self, data: &ColorsChosen) -> Result<(), XAddErr>;
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
    fn xadd(&self, data: &ColorsChosen) -> Result<(), XAddErr> {
        let ser_bytes_result = bincode::serialize(&data);

        if let Ok(bytes) = ser_bytes_result {
            let mut m: BTreeMap<&str, &[u8]> = BTreeMap::new();
            m.insert(MAP_KEY, &bytes);
            if let Ok(mut conn) = self.get_connection() {
                conn.xadd_maxlen_map(
                    topics::COLORS_CHOSEN,
                    StreamMaxlen::Approx(MAX_LEN),
                    AUTO_ID,
                    m,
                )
                .map_err(|e| XAddErr::Redis(e))
            } else {
                Err(XAddErr::Conn)
            }
        } else {
            Err(XAddErr::Ser)
        }
    }
}
