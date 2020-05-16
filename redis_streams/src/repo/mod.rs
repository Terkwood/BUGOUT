use super::XReadEntryId;
use redis::Client;
use redis::Commands;
use std::collections::HashMap;

pub fn fetch_entry_ids<A: Default>(
    client: &Client,
    redis_key: &str,
    deser: Box<dyn Fn(HashMap<String, String>) -> A>,
) -> Result<A, EntryIdRepoErr> {
    if let Ok(mut conn) = client.get_connection() {
        let found: Result<HashMap<String, String>, _> = conn.hgetall(redis_key);
        Ok(if let Ok(hash) = found {
            deser(hash)
        } else {
            A::default()
        })
    } else {
        Err(EntryIdRepoErr)
    }
}

pub fn update_entry_id<B>(
    entry_id_type: B,
    entry_id: XReadEntryId,
    client: &Client,
    redis_key: &str,
    hash_field: Box<dyn Fn(B) -> String>,
) -> Result<(), EntryIdRepoErr> {
    if let Ok(mut conn) = client.get_connection() {
        conn.hset(redis_key, hash_field(entry_id_type), entry_id.to_string())
            .map_err(|_| EntryIdRepoErr)
    } else {
        Err(EntryIdRepoErr)
    }
}

pub struct EntryIdRepoErr;
