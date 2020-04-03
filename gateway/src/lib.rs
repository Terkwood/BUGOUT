#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate crossbeam;
extern crate crossbeam_channel;
extern crate dotenv;
extern crate env_logger;
extern crate envy;
extern crate harsh;
extern crate log;
extern crate mio_extras;
extern crate r2d2_redis;
extern crate rand;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate time;
extern crate ws;

pub mod backend;
pub mod backend_commands;
pub mod backend_events;
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

pub const EMPTY_SHORT_UUID: &str = "        ";
pub fn short_uuid(uuid: Uuid) -> String {
    uuid.to_string()[..8].to_string()
}

pub const FULL_BOARD_SIZE: u8 = 19;
pub const SMALL_BOARD_SIZE: u8 = 9;
