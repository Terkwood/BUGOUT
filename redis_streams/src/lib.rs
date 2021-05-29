mod consumer_group_opts;
mod sorted_streams;
mod stream_handler;

pub use anyhow;
pub use consumer_group_opts::*;
pub use sorted_streams::*;
pub use stream_handler::*;

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct XId {
    pub millis_time: u64,
    pub seq_no: u64,
}
impl Default for XId {
    fn default() -> Self {
        XId {
            millis_time: 0,
            seq_no: 0,
        }
    }
}

impl std::str::FromStr for XId {
    type Err = StreamDeserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            Err(StreamDeserError)
        } else {
            let millis_time = parts[0].parse::<u64>()?;
            let seq_no = parts[1].parse::<u64>()?;
            Ok(XId {
                millis_time,
                seq_no,
            })
        }
    }
}

impl std::fmt::Display for XId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.millis_time, self.seq_no)
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
        assert_eq!(XId::default().to_string(), "0-0".to_string())
    }
}
