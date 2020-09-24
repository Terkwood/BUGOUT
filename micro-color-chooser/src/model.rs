use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Serialize)]
pub enum Color {
    Black,
    White,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum ColorPref {
    Black,
    White,
    Any,
}

pub struct SessionColorPref(pub SessionId, pub ColorPref);

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClientId(pub Uuid);
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionId(pub Uuid);
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameId(pub Uuid);
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EventId(pub Uuid);
