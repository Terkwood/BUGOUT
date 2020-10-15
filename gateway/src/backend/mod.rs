pub mod commands;
mod doubler;
pub mod events;
mod start;

use crate::backend::events::{BackendEvents, KafkaShutdownEvent};
use crate::idle_status::KafkaActivityObserved;
pub use doubler::double_commands;
pub use start::{start_all, BackendInitOptions};
