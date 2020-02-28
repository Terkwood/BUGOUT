pub mod topics;
use crate::repo::*;
use redis_conn_pool::Pool;
use redis_key::HashKeyProvider;
pub use topics::StreamTopics;
pub fn process(topics: StreamTopics, components: &crate::Components) {
    println!("Processing {:#?}", topics);
}
