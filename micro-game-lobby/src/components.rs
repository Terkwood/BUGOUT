use crate::repo::{EntryIdRepo, RedisEntryIdRepo};
use crate::stream::{RedisXReader, XReader};

use redis_conn_pool::{Pool, RedisHostUrl};
use std::sync::Arc;

pub struct Components {
    pub entry_id_repo: Box<dyn EntryIdRepo>,
    pub xreader: Box<dyn XReader>,
}

impl Default for Components {
    fn default() -> Self {
        let pool = redis_conn_pool::create(RedisHostUrl::default());
        let arc_pool = Arc::new(pool);
        Components {
            entry_id_repo: Box::new(RedisEntryIdRepo {
                pool: arc_pool.clone(),
                key_provider: (),
            }),
            xreader: Box::new(RedisXReader { pool: arc_pool }),
        }
    }
}
