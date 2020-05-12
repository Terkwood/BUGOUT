use crate::api::*;
use redis::Client;
use std::sync::Arc;

pub trait XAdd {
    fn xadd(&self, data: StreamOutput) -> Result<(), XAddErr>;
}

#[derive(Debug, Clone)]
pub enum XAddErr {
    Some,
    Other,
}

impl XAdd for Arc<Client> {
    fn xadd(&self, data: StreamOutput) -> Result<(), XAddErr> {
        todo!()
    }
}

pub enum StreamOutput {
    WFO(WaitForOpponent),
    GR(GameReady),
    PGR(PrivateGameRejected),
}
