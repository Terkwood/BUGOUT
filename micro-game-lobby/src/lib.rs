extern crate bincode;
extern crate community_redis_streams;
extern crate crossbeam_channel;
extern crate log;
extern crate redis;
extern crate serde;
extern crate serde_derive;

pub mod api;
pub mod components;
mod game_lobby;
mod repo;
pub mod stream;
pub mod topics;

use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameId(pub Uuid);
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Hash)]
pub struct ClientId(pub Uuid);
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct EventId(pub Uuid);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
}

pub const PUBLIC_GAME_BOARD_SIZE: u16 = 19;

impl GameId {
    pub fn new() -> Self {
        GameId(Uuid::new_v4())
    }
}
impl EventId {
    pub fn new() -> Self {
        EventId(Uuid::new_v4())
    }
}
