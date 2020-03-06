extern crate bincode;
extern crate serde;
extern crate serde_derive;

mod components;
mod entry_id_repo;
mod game_lobby;
pub mod stream;
mod topics;

use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StreamTopics {}
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameId(pub Uuid);
#[derive(Debug, Clone)]
pub struct ClientId(pub Uuid);
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);
#[derive(Debug, Clone)]
pub struct EventId(pub Uuid);
impl EventId {
    pub fn new() -> Self {
        EventId(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
}
