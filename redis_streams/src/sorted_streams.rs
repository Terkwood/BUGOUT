use crate::{
    consumer_group::{ConsumerGroup, Message},
    XId,
};
use anyhow::{Context, Result};
use std::collections::HashMap;

trait SortedStreams {
    fn consume(&self) -> Result<()>;
}

impl<'a, F> SortedStreams for Vec<ConsumerGroup<'a, F>>
where
    F: FnMut(XId, &Message) -> Result<()>,
{
    fn consume(&self) -> Result<()> {
        let mut unacked = Unacknowledged::default();
        for _consumer_group in self {
            todo!()
        }

        Ok(())
    }
}

/// Track unacknowledged messages by stream name
struct Unacknowledged(pub HashMap<String, Vec<XId>>);
impl Default for Unacknowledged {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_consume() {
        todo!()
    }
}
