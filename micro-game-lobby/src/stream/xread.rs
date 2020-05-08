use super::topics;
use crate::api::*;
use crate::repo::AllEntryIds;
use log::{trace, warn};
use redis_conn_pool::redis;
use redis_conn_pool::Pool;
use redis_streams::XReadEntryId;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

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
        let xrr = redis::cmd("XREAD")
            .arg("BLOCK")
            .arg(&BLOCK_MSEC.to_string())
            .arg("STREAMS")
            /*.arg(todo!())
            .arg(todo!("topic str 1"))
            .arg(todo!("eid 0 to string"))
            .arg(todo!("eid 1 to string"))*/
            .query::<XReadResult>(&mut *conn)?;
        let unsorted: HashMap<XReadEntryId, StreamData> = todo!("deser");
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
    }
}

fn deser(xread_result: XReadResult) -> HashMap<XReadEntryId, StreamData> {
    todo!()
}

#[derive(Clone, Debug)]
pub enum StreamData {
    FPG(FindPublicGame),
    CG(CreateGame),
    JPG(JoinPrivateGame),
}
