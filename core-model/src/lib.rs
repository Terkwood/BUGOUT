use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GameId(pub Uuid);

impl GameId {
    pub fn now() -> Self {
        Self(Uuid::new_v4())
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ReqId(pub Uuid);

impl ReqId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct EventId(pub Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
