use super::XReadEntryId;
use redis_conn_pool::redis::Commands;
use redis_conn_pool::Pool;
use std::collections::HashMap;

pub fn fetch_all<A: Default>(
    pool: &Pool,
    provide_key: Box<dyn Fn() -> String>,
    deser: Box<dyn Fn(HashMap<String, String>) -> A>,
) -> Result<A, EntryIdRepoErr> {
    if let Ok(mut conn) = pool.get() {
        let found: Result<HashMap<String, String>, _> = conn.hgetall(provide_key());
        Ok(if let Ok(hash) = found {
            deser(hash)
        } else {
            A::default()
        })
    } else {
        Err(EntryIdRepoErr)
    }
}

pub fn update<B>(
    entry_id_type: B,
    entry_id: XReadEntryId,
    pool: &Pool,
    provide_key: Box<dyn Fn() -> String>,
    hash_field: Box<dyn Fn(B) -> String>,
) -> Result<(), EntryIdRepoErr> {
    let mut conn = pool.get().expect("redis pool");
    conn.hset(
        provide_key(),
        hash_field(entry_id_type),
        entry_id.to_string(),
    )
    .map_err(|_| EntryIdRepoErr)
}

pub struct EntryIdRepoErr;
