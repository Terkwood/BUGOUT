use crate::kafka_events::*;
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

/// The running status of an expensive container host
///
/// - Idle (since when)
/// - Booting (since when)
/// - Awake (you may proceed to have fun)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum IdleStatus {
    Idle(DateTime<Utc>),
    Booting(DateTime<Utc>),
    Online,
}

pub fn start_monitor(events_in: crossbeam::Sender<KafkaEvents>) {
    println!("Hello Please")
}
