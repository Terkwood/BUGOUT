use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GameId(pub Uuid);
#[cfg(test)]
impl GameId {
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct SessionId(pub Uuid);

#[cfg(test)]
impl SessionId {
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ReqId(pub Uuid);
#[cfg(test)]
impl ReqId {
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct EventId(pub Uuid);
