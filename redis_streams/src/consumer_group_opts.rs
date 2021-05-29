use crate::*;
#[derive(Debug)]
pub struct ConsumerGroupOpts {
    pub count: Option<usize>,
    pub group: Group,
    pub block_ms: usize,
}

impl ConsumerGroupOpts {
    pub fn new(group: Group) -> Self {
        Self {
            count: None,
            group,
            block_ms: 5_000,
        }
    }

    /// Maximum number of message to read from the stream in one batch
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    /// Name of the group and consumer. Enables Redis group consumer behavior if
    /// specified
    pub fn group(mut self, group_name: &str, consumer_name: &str) -> Self {
        self.group = Group {
            group_name: group_name.to_string(),
            consumer_name: consumer_name.to_string(),
        };
        self
    }

    /// Maximum ms duration to block waiting for messages.
    pub fn block_ms(mut self, timeout_ms: usize) -> Self {
        self.block_ms = timeout_ms;
        self
    }
}
