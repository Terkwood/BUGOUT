mod doubler;
mod start;

use crate::backend_events::{BackendEvents, KafkaShutdownEvent};
use crate::idle_status::KafkaActivityObserved;
pub use doubler::double_commands;
pub use start::{start_all, BackendInitOptions};
