mod topics;
mod xadd;
mod xread;

pub use xadd::XAdd;
pub use xread::XRead;

use crate::api::*;
use crate::components::*;
use crate::model::*;

#[derive(Clone, Debug)]
pub enum StreamData {
    PH(ProvideHistory),
    GS(GameId, GameState),
}

pub fn process(components: &Components) {
    loop {
        todo!()
    }
}

pub fn create_consumer_group(client: &redis::Client) {
    todo!()
}
