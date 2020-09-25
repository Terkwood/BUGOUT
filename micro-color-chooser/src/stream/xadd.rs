use crate::api::ColorsChosen;
use redis::Client;
use std::rc::Rc;
pub trait XAdd {
    fn xadd(&self, data: ColorsChosen) -> Result<(), XAddErr>;
}
#[derive(Debug)]
pub enum XAddErr {
    Redis(redis::RedisError),
    Ser,
    Conn,
}

impl XAdd for Rc<Client> {
    fn xadd(&self, data: ColorsChosen) -> Result<(), XAddErr> {
        todo!()
    }
}
