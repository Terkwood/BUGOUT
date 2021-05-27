use crate::consumer_group::{ConsumerGroup, Message};
use anyhow::{Context, Result};

trait SortedStreams {
    fn consume(&self) -> Result<()>;
}

impl<'a, F> SortedStreams for Vec<ConsumerGroup<'a, F>>
where
    F: FnMut(&str, &Message) -> Result<()>,
{
    fn consume(&self) -> Result<()> {
        for _consumer_group in self {
            todo!()
        }

        Ok(())
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
