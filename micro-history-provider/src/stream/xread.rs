use crate::stream::StreamInput;
use redis::Client;
use redis_streams::XReadEntryId;
use std::rc::Rc;
pub trait XRead {
    fn xread_sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, StreamReadErr>;
    fn xack_prov_hist(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
    fn xack_game_states(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr>;
}
impl XRead for Rc<Client> {
    fn xread_sorted(&self) -> Result<Vec<(XReadEntryId, StreamInput)>, StreamReadErr> {
        todo!()
    }

    fn xack_prov_hist(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        todo!()
    }

    fn xack_game_states(&self, ids: &[XReadEntryId]) -> Result<(), StreamAckErr> {
        todo!()
    }
}

#[derive(Debug)]
pub enum StreamReadErr {
    Deser(StreamDeserErr),
    Other,
}
#[derive(Debug)]
pub struct StreamDeserErr;
#[derive(Debug)]
pub struct StreamAckErr;
