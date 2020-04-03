use futures::executor::block_on;

use crate::backend_commands::{BackendCommands, SessionCommand};
use crate::backend_events::{BackendEvents, KafkaShutdownEvent};
use crate::idle_status::KafkaActivityObserved;
use crate::kafka_io;
use crate::redis_io;

use crossbeam_channel::{select, unbounded, Receiver, Sender};
use std::thread;

pub mod repo;

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

pub struct BackendInitOptions {
    pub backend_events_in: Sender<BackendEvents>,
    pub shutdown_in: Sender<KafkaShutdownEvent>,
    pub kafka_activity_in: Sender<KafkaActivityObserved>,
    pub session_commands_out: Receiver<SessionCommand>,
    pub client_repo: Box<dyn repo::ClientBackendRepo>,
}

pub fn start_all(opts: BackendInitOptions) {
    let (kafka_commands_in, kafka_commands_out): (
        Sender<BackendCommands>,
        Receiver<BackendCommands>,
    ) = unbounded();

    let (redis_commands_in, redis_commands_out): (
        Sender<BackendCommands>,
        Receiver<BackendCommands>,
    ) = unbounded();

    thread::spawn(move || redis_io::command_writer::start(redis_commands_out));
    let bei = opts.backend_events_in.clone();
    thread::spawn(move || redis_io::stream::process(bei));

    let soc = opts.session_commands_out.clone();
    thread::spawn(move || split(soc, kafka_commands_in, redis_commands_in));

    block_on(kafka_io::start(
        opts.backend_events_in.clone(),
        opts.shutdown_in.clone(),
        opts.kafka_activity_in.clone(),
        kafka_commands_out.clone(),
    ))
}

fn split(
    session_commands_out: Receiver<SessionCommand>,
    _kafka_commands_in: Sender<BackendCommands>,
    _redis_commands_in: Sender<BackendCommands>,
) {
    loop {
        select! {
            recv(session_commands_out) -> _ => todo!()
        }
    }
}
