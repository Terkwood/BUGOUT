use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum ColorPref {
    Black,
    White,
    Any,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClientId(pub Uuid);
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionId(pub Uuid);
