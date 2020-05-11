use super::topics::{CREATE_GAME, FIND_PUBLIC_GAME, JOIN_PRIVATE_GAME};
use crate::api::*;
use crate::community_redis_streams::StreamCommands;
use crate::repo::AllEntryIds;
use redis_conn_pool::Pool;
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::sync::Arc;

const BLOCK_MSEC: u32 = 5000;

pub type XReadResult = Vec<HashMap<String, Vec<HashMap<String, redis::Value>>>>;

/// xread_sorted performs a redis xread then sorts the results
///
/// entry_ids: the minimum entry ids from which to read
pub trait XReader: Send + Sync {
    fn xread_sorted(
        &self,
        entry_ids: AllEntryIds,
    ) -> Result<Vec<(XReadEntryId, StreamData)>, redis::RedisError>;
}

pub struct RedisXReader {
    pub pool: Arc<Pool>,
}

impl XReader for RedisXReader {
    fn xread_sorted(
        &self,
        entry_ids: AllEntryIds,
    ) -> Result<std::vec::Vec<(XReadEntryId, StreamData)>, redis::RedisError> {
        let mut conn = self.pool.get().unwrap();

        let mut conn = self.pool.get().expect("pool");
        conn.xread(todo!(), todo!());
        todo!()
        /*
            let xrr = redis::cmd("XREAD")
                .arg("BLOCK")
                .arg(&BLOCK_MSEC.to_string())
                .arg("STREAMS")
                .arg(FIND_PUBLIC_GAME)
                .arg(CREATE_GAME)
                .arg(JOIN_PRIVATE_GAME)
                .arg(entry_ids.find_public_game.to_string())
                .arg(entry_ids.create_game.to_string())
                .arg(entry_ids.join_private_game.to_string())
                .query::<XReadResult>(&mut *conn)?;
            let unsorted: HashMap<XReadEntryId, StreamData> = deser(xrr);
            let sorted_keys: Vec<XReadEntryId> = {
                let mut ks: Vec<XReadEntryId> = unsorted.keys().copied().collect();
                ks.sort();
                ks
            };
            let mut answer = vec![];
            for sk in sorted_keys {
                if let Some(data) = unsorted.get(&sk) {
                    answer.push((sk, data.clone()))
                }
            }
            Ok(answer)
        }*/
    }
}

fn deser(_xread_result: XReadResult) -> HashMap<XReadEntryId, StreamData> {
    todo!()
}

#[derive(Clone, Debug)]
pub enum StreamData {
    FPG(FindPublicGame),
    CG(CreateGame),
    JPG(JoinPrivateGame),
}
