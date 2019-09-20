use std::time::SystemTime;

use serde_derive::{Deserialize, Serialize};

pub struct KafkaActivity {
    pub topic: String,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShutdownCommand(pub SystemTime);

impl ShutdownCommand {
    pub fn new() -> ShutdownCommand {
        ShutdownCommand(SystemTime::now())
    }

    pub fn as_millis(&self) -> u128 {
        self.0
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Default::default())
            .as_millis()
    }
}
