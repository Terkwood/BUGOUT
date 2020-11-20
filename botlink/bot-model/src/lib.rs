pub mod api;

use serde_derive::{Deserialize, Serialize};

/// This enum represents various Go-playing programs with different
/// difficulties and time constraints.  Currently uses only KataGo, but
/// in https://github.com/Terkwood/BUGOUT/issues/322 and
/// https://github.com/Terkwood/BUGOUT/issues/440 we plan to investigate
/// using other programs to provide other computer-controlled
/// opponents with their own difficulty settings.
#[derive(Copy, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
pub enum Bot {
    KataGoOneStar,
    KataGoTwoStars,
    KataGoThreeStars,
    KataGoFourStars,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlphaNumCoord(pub char, pub u16);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_kata_instant_json() {
        let input = Bot::KataGoInstant;
        let json = serde_json::to_string(&input).expect("to_string");
        assert_eq!(json, "\"KataGoInstant\"")
    }
    #[test]
    fn test_kata_full_json() {
        let input = Bot::KataGoFullStrength;
        let json = serde_json::to_string(&input).expect("to_string");
        assert_eq!(json, "\"KataGoFullStrength\"")
    }
}
