use crate::stream::StreamInput;
use redis::Client;
use redis_streams::XReadEntryId;
use std::rc::Rc;
pub trait XRead {
    fn xread_sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, redis::RedisError>;
}
impl XRead for Rc<Client> {
    fn xread_sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, redis::RedisError> {
        todo!()
    }
}
