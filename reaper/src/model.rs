use std::time::SystemTime;

use serde_derive::{Deserialize, Serialize};

pub struct KafkaActivity {
    pub topic: String,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShutdownCommand(pub u128);

impl ShutdownCommand {
    pub fn new() -> ShutdownCommand {
        ShutdownCommand(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Default::default())
                .as_millis(),
        )
    }
}
