use crate::api::HistoryProvided;
use redis::Client;
use std::rc::Rc;
pub trait XAdd {
    fn xadd(&self, data: HistoryProvided) -> Result<(), XAddErr>;
}
impl XAdd for Rc<Client> {
    fn xadd(&self, data: HistoryProvided) -> Result<(), XAddErr> {
        todo!()
    }
}

#[derive(Debug)]
pub enum XAddErr {
    Redis(redis::RedisError),
    Ser,
    Conn,
}
