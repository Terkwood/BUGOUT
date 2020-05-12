use redis::Client;
use std::sync::Arc;
pub trait XAdd {}
impl XAdd for Arc<Client> {}
