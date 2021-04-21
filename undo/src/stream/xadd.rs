use super::topics::*;
use super::StreamOutput;
use redis::Client;
use redis::{streams::StreamMaxlen, Commands};
use std::collections::BTreeMap;
use std::rc::Rc;

pub trait XAdd {
    fn xadd(&self, data: StreamOutput) -> Result<(), XAddErr>;
}
#[derive(Debug)]
pub enum XAddErr {
    Redis(redis::RedisError),
    Ser,
    Conn,
}

impl XAdd for Rc<Client> {
    fn xadd(&self, data: StreamOutput) -> Result<(), XAddErr> {
        todo!()
    }
}
