use crate::api::*;
use redis::Client;
use std::rc::Rc;

pub trait XAdd {
    fn xadd(&self, data: StreamOutput) -> Result<(), XAddErr>;
}

#[derive(Debug, Clone)]
pub enum XAddErr {
    Some,
    Other,
}

impl XAdd for Rc<Client> {
    fn xadd(&self, _data: StreamOutput) -> Result<(), XAddErr> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamOutput {
    WFO(WaitForOpponent),
    GR(GameReady),
    PGR(PrivateGameRejected),
}
