#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct XReadEntryId {
    pub millis_time: u64,
    pub seq_no: u64,
}
impl Default for XReadEntryId {
    fn default() -> Self {
        XReadEntryId {
            millis_time: 0,
            seq_no: 0,
        }
    }
}

impl XReadEntryId {
    pub fn from_str(s: &str) -> Result<XReadEntryId, StreamDeserError> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            Err(StreamDeserError)
        } else {
            let millis_time = parts[0].parse::<u64>()?;
            let seq_no = parts[1].parse::<u64>()?;
            Ok(XReadEntryId {
                millis_time,
                seq_no,
            })
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}-{}", self.millis_time, self.seq_no)
    }
}

#[derive(Debug)]
pub struct StreamDeserError;
impl From<uuid::Error> for StreamDeserError {
    fn from(_: uuid::Error) -> StreamDeserError {
        StreamDeserError
    }
}
impl From<std::num::ParseIntError> for StreamDeserError {
    fn from(_: std::num::ParseIntError) -> StreamDeserError {
        StreamDeserError
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xread_entry_id_default_string() {
        assert_eq!(XReadEntryId::default().to_string(), "0-0".to_string())
    }
}
