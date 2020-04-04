pub mod client_repo;
pub mod game_repo;
pub mod session_repo;

mod repo_err;
mod split;
mod start;

pub use client_repo::ClientBackendRepo;
pub use game_repo::GameBackendRepo;
pub use repo_err::*;
pub use session_repo::SessionBackendRepo;
pub use split::split_commands;
pub use start::{start_all, BackendInitOptions};

use crate::backend_commands::SessionCommands;
use crate::backend_events::{BackendEvents, KafkaShutdownEvent};
use crate::idle_status::KafkaActivityObserved;

use crossbeam_channel::{Receiver, Sender};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Backend {
    RedisStreams,
    Kafka,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Backend::RedisStreams => "rs",
                Backend::Kafka => "k",
            }
        )
    }
}
