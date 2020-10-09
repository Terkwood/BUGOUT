#[macro_use]
extern crate lazy_static;

pub mod backend;
pub mod backend_commands;
pub mod backend_events;
pub mod channels;
pub mod compact_ids;
pub mod env;
pub mod idle_status;
pub mod kafka_io;
pub mod redis_io;
pub mod router;
pub mod websocket;

mod client_commands;
mod client_events;
mod logging;
mod model;
mod topics;
mod wakeup;
use uuid::Uuid;

/// A spacer used for formatting
pub const EMPTY_SHORT_UUID: &str = "        ";

/// The first 8 digits of a UUID
pub fn short_uuid(uuid: Uuid) -> String {
    uuid.to_string()[..8].to_string()
}

pub const FULL_BOARD_SIZE: u8 = 19;
