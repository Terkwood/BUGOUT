pub mod api;

use serde_derive::{Deserialize, Serialize};

#[derive(Copy, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Max,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlphaNumCoord(pub char, pub u16);
