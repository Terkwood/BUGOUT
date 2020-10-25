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

#[test]
fn test_difficulty_json() {
    let input = Difficulty::Max;
    let json = serde_json::to_string(&input).expect("to_string");
    assert_eq!(json, "\"Max\"")
}
