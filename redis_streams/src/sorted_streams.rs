use crate::consumer_group::{ConsumerGroup, Message};
use anyhow::{Context, Result};

trait SortedStreams {
    fn consume() -> Result<()>;
}

impl<'a, F> SortedStreams for Vec<ConsumerGroup<'a, F>> where F: FnMut(&str, &Message) -> Result<()> {
    fn consume() -> Result<()> {
        todo!()
    }
}
