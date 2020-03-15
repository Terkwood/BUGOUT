use serde_derive::{Deserialize, Serialize};
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CoordOutOfRange;

impl From<std::num::ParseIntError> for CoordOutOfRange {
    fn from(_: std::num::ParseIntError) -> Self {
        CoordOutOfRange
    }
}

#[derive(Debug)]
pub enum KataGoParseErr {
    UuidErr(uuid::Error),
    WrongFormat,
    Coord,
}
impl From<uuid::Error> for KataGoParseErr {
    fn from(u: uuid::Error) -> Self {
        KataGoParseErr::UuidErr(u)
    }
}
impl From<CoordOutOfRange> for KataGoParseErr {
    fn from(_: CoordOutOfRange) -> Self {
        KataGoParseErr::Coord
    }
}
