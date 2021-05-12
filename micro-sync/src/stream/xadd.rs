use super::topics;
use crate::move_model::{Coord, MakeMove};
use redis::{streams::StreamMaxlen, Client, Commands};
use std::collections::BTreeMap;
use std::rc::Rc;
use sync_model::api::{HistoryProvided, SyncReply};

pub trait XAdd {
    fn add_history_provided(&self, data: HistoryProvided) -> Result<(), XAddErr>;
    fn add_sync_reply(&self, data: SyncReply) -> Result<(), XAddErr>;
    fn add_make_move(&self, data: MakeMove) -> Result<(), XAddErr>;
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
    fn add_history_provided(&self, data: HistoryProvided) -> Result<(), XAddErr> {
        let ser_bytes_result = bincode::serialize(&data);

        if let Ok(bytes) = ser_bytes_result {
            let mut m: BTreeMap<&str, &[u8]> = BTreeMap::new();
            m.insert(MAP_KEY, &bytes);
            if let Ok(mut conn) = self.get_connection() {
                conn.xadd_maxlen_map(
                    topics::HISTORY_PROVIDED,
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

    fn add_sync_reply(&self, data: SyncReply) -> Result<(), XAddErr> {
        let ser_bytes_result = bincode::serialize(&data);

        if let Ok(bytes) = ser_bytes_result {
            let mut m: BTreeMap<&str, &[u8]> = BTreeMap::new();
            m.insert(MAP_KEY, &bytes);
            if let Ok(mut conn) = self.get_connection() {
                conn.xadd_maxlen_map(
                    topics::SYNC_REPLY,
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

    fn add_make_move(&self, data: MakeMove) -> Result<(), XAddErr> {
        // See _clean_add_make_move to satisfy #363
        // and get rid of this complex data format
        if let Ok(mut conn) = self.get_connection() {
            let mut cmd = redis::cmd("XADD");
            cmd.arg(topics::MAKE_MOVE)
                .arg("MAXLEN")
                .arg("~")
                .arg("1000")
                .arg("*")
                .arg("game_id")
                .arg(data.game_id.0.to_string())
                .arg("player")
                .arg(data.player.to_string())
                .arg("req_id")
                .arg(data.req_id.0.to_string());
            if let Some(Coord { x, y }) = data.coord {
                cmd.arg("coord_x").arg(x).arg("coord_y").arg(y);
            }
            cmd.query::<String>(&mut conn)
                .map(|_| ())
                .map_err(|e| XAddErr::Redis(e))
        } else {
            Err(XAddErr::Conn)
        }
    }
}

/// This can be used to satisfy https://github.com/Terkwood/BUGOUT/issues/363
fn _clean_add_make_move(client: &Client, data: MakeMove) -> Result<(), XAddErr> {
    let ser_bytes_result = bincode::serialize(&data);

    if let Ok(bytes) = ser_bytes_result {
        let mut m: BTreeMap<&str, &[u8]> = BTreeMap::new();
        m.insert(MAP_KEY, &bytes);
        if let Ok(mut conn) = client.get_connection() {
            conn.xadd_maxlen_map(topics::MAKE_MOVE, StreamMaxlen::Approx(MAX_LEN), AUTO_ID, m)
                .map_err(|e| XAddErr::Redis(e))
        } else {
            Err(XAddErr::Conn)
        }
    } else {
        Err(XAddErr::Ser)
    }
}
